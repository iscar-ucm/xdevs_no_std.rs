use crate::{atomic::Atomic, port::UnsafePort};

/// Interface for simulating DEVS models. All DEVS models must implement this trait.
pub unsafe trait Simulator {
    /// Returns the time for the last state transition of the inner DEVS [`Component`].
    fn get_t_last(&self) -> f64;

    /// Returns the time for the next state transition of the inner DEVS [`Component`].
    fn get_t_next(&self) -> f64;

    /// Sets the tine for the last and next state transitions of the inner DEVS [`Component`].
    fn set_sim_t(&mut self, t_last: f64, t_next: f64);

    /// Removes all the messages from all the ports.
    fn clear_ports(&mut self);

    /// It starts the simulation, setting the initial time to t_start.
    fn start(&mut self, t_start: f64);

    /// It stops the simulation, setting the last time to t_stop.
    fn stop(&mut self, t_stop: f64);

    /// Executes output functions and propagates messages according to ICs and EOCs.
    fn collection(&mut self, t: f64);

    /// Propagates messages according to EICs and executes model transition functions.
    /// Returns the time for the next state transition of the inner DEVS model.
    fn transition(&mut self, t: f64) -> f64;
}

unsafe impl<T: Atomic> Simulator for T {
    #[inline]
    fn get_t_last(&self) -> f64 {
        self.get_component().t_last
    }

    #[inline]
    fn get_t_next(&self) -> f64 {
        self.get_component().t_next
    }

    #[inline]
    fn set_sim_t(&mut self, t_last: f64, t_next: f64) {
        self.get_component_mut().set_sim_t(t_last, t_next);
    }

    #[inline]
    fn clear_ports(&mut self) {
        self.get_component_mut().clear_ports();
    }

    #[inline]
    fn start(&mut self, t_start: f64) {
        let state = self.get_state_mut();
        Self::start(state);
        let ta = Self::ta(state);
        self.set_sim_t(t_start, t_start + ta);
    }

    #[inline]
    fn stop(&mut self, t_stop: f64) {
        self.set_sim_t(t_stop, f64::INFINITY);
        Self::stop(self.get_state_mut());
    }

    #[inline]
    fn collection(&mut self, t: f64) {
        if t >= self.get_t_next() {
            let (state, component) = self.divide_mut();
            Self::lambda(state, &mut component.output)
        }
    }

    #[inline]
    fn transition(&mut self, t: f64) -> f64 {
        let (state, component) = self.divide_mut();
        let input = &component.input;
        if !input.is_empty() {
            if t >= component.t_next {
                Self::delta_conf(state, input);
            } else {
                let e = t - component.t_last;
                Self::delta_ext(state, e, input);
            }
        } else if t >= component.t_next {
            Self::delta_int(state);
        } else {
            return component.t_next;
        }
        let t_next = t + Self::ta(state);
        self.set_sim_t(t, t_next);
        self.clear_ports();
        t_next
    }
}
