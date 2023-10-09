use crate::atomic::Atomic;

/// Interface for simulating DEVS models. All DEVS models must implement this trait.
///
/// # Safety
///
/// This trait must be implemented via the [`atomic!`] and [`coupled!`] macros. Do not implement it manually.
pub unsafe trait Simulator {
    /// It starts the simulation, setting the initial time to t_start.
    /// It returns the time for the next state transition of the inner DEVS model.
    fn start(&mut self, t_start: f64) -> f64;

    /// It stops the simulation, setting the last time to t_stop.
    fn stop(&mut self, t_stop: f64);

    /// Executes output functions and propagates messages according to ICs and EOCs.
    /// Internally, it checks that the model is imminent before executing.
    fn lambda(&mut self, t: f64);

    /// Propagates messages according to EICs and executes model transition functions.
    /// It also clears all the input and output ports.
    /// Internally, it checks that the model is imminent before executing.
    /// Finally, it returns the time for the next state transition of the inner DEVS model.
    fn delta(&mut self, t: f64) -> f64;
}

unsafe impl<T: Atomic> Simulator for T {
    #[inline]
    fn start(&mut self, t_start: f64) -> f64 {
        let (state, component) = self.divide_mut();
        Self::start(state);
        let t_next = t_start + Self::ta(state);
        component.set_sim_t(t_start, t_next);
        t_next
    }

    #[inline]
    fn stop(&mut self, t_stop: f64) {
        let (state, component) = self.divide_mut();
        component.set_sim_t(t_stop, f64::INFINITY);
        Self::stop(state);
    }

    #[inline]
    fn lambda(&mut self, t: f64) {
        let (state, component) = self.divide_mut();
        if t >= component.t_next {
            Self::lambda(state, &mut component.output)
        }
    }

    #[inline]
    fn delta(&mut self, t: f64) -> f64 {
        let (state, component) = self.divide_mut();
        if !component.is_input_empty() {
            if t >= component.t_next {
                Self::delta_conf(state, &component.input);
            } else {
                let e = t - component.t_last;
                Self::delta_ext(state, e, &component.input);
            }
        } else if t >= component.t_next {
            Self::delta_int(state);
        } else {
            return component.t_next;
        }
        component.clear_ports();

        let t_next = t + Self::ta(state);
        component.set_sim_t(t, t_next);
        t_next
    }
}
