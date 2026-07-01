use crate::{AtomicKind, Component, Duration};

/// Interface for DEVS atomic models. All DEVS atomic models must implement this trait.
pub trait Atomic: Component<Kind = AtomicKind> {
    /// Method for performing any operation before simulating. By default, it does nothing.
    #[allow(unused_variables)]
    #[inline(always)]
    fn start(&mut self) {}

    /// Method for performing any operation after simulating. By default, it does nothing.
    #[allow(unused_variables)]
    #[inline(always)]
    fn stop(&mut self) {}

    /// Internal transition function. It modifies the state of the model when an internal event happens.
    fn delta_int(&mut self);

    /// External transition function. It modifies the state of the model when an external event happens.
    /// The time elapsed since the last state transition is `elapsed`.
    fn delta_ext(&mut self, elapsed: Duration, input: &Self::Input);

    /// Confluent transition function. It modifies the state of the model when an external and an internal event occur simultaneously.
    /// By default, it calls [`Atomic::delta_int`] and [`Atomic::delta_ext`] with `elapsed = 0`, in that order.
    #[inline(always)]
    fn delta_conf(&mut self, input: &Self::Input) {
        Self::delta_int(self);
        Self::delta_ext(self, Duration::from_secs(0), input);
    }

    /// Output function. It triggers output events when an internal event is about to happen.
    fn lambda(&self, output: &mut Self::Output);

    /// Time advance function. It returns the time until the next internal event happens.
    fn ta(&self) -> Duration;
}

impl<T: Atomic> Atomic for &mut T {
    #[inline(always)]
    fn start(&mut self) {
        T::start(self)
    }

    #[inline(always)]
    fn stop(&mut self) {
        T::stop(self)
    }

    #[inline(always)]
    fn delta_int(&mut self) {
        T::delta_int(self)
    }

    #[inline(always)]
    fn delta_ext(&mut self, elapsed: Duration, input: &Self::Input) {
        T::delta_ext(self, elapsed, input)
    }

    #[inline(always)]
    fn delta_conf(&mut self, input: &Self::Input) {
        T::delta_conf(self, input)
    }

    #[inline(always)]
    fn lambda(&self, output: &mut Self::Output) {
        T::lambda(self, output)
    }

    #[inline(always)]
    fn ta(&self) -> Duration {
        T::ta(self)
    }
}

#[cfg(feature = "alloc")]
impl<T: Atomic> Atomic for alloc::boxed::Box<T> {
    #[inline(always)]
    fn start(&mut self) {
        T::start(self)
    }

    #[inline(always)]
    fn stop(&mut self) {
        T::stop(self)
    }

    #[inline(always)]
    fn delta_int(&mut self) {
        T::delta_int(self)
    }

    #[inline(always)]
    fn delta_ext(&mut self, elapsed: Duration, input: &Self::Input) {
        T::delta_ext(self, elapsed, input)
    }

    #[inline(always)]
    fn delta_conf(&mut self, input: &Self::Input) {
        T::delta_conf(self, input)
    }

    #[inline(always)]
    fn lambda(&self, output: &mut Self::Output) {
        T::lambda(self, output)
    }

    #[inline(always)]
    fn ta(&self) -> Duration {
        T::ta(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Atomic, AtomicKind, Bag, Component, Duration, Port};

    struct CallTracker {
        start: bool,
        stop: bool,
        delta_int: bool,
        delta_ext_elapsed: Duration,
        ta_val: Duration,
    }

    impl Component for CallTracker {
        type Kind = AtomicKind;
        type Input = ();
        type Output = Port<usize, 1>;
    }

    impl Atomic for CallTracker {
        fn start(&mut self) {
            self.start = true;
        }
        fn stop(&mut self) {
            self.stop = true;
        }
        fn delta_int(&mut self) {
            self.delta_int = true;
        }
        fn delta_ext(&mut self, elapsed: Duration, _input: &Self::Input) {
            self.delta_ext_elapsed = elapsed;
        }
        fn lambda(&self, output: &mut Self::Output) {
            let _ = output.add_value(99);
        }
        fn ta(&self) -> Duration {
            self.ta_val
        }
    }

    #[test]
    fn all_methods_raw() {
        let mut model = CallTracker {
            start: false,
            stop: false,
            delta_int: false,
            delta_ext_elapsed: Duration::MAX,
            ta_val: Duration::from_secs(7),
        };
        let mut output = Port::<usize, 1>::new();

        model.start();
        assert!(model.start, "start happened");

        model.lambda(&mut output);
        assert_eq!(output.get_values(), &[99], "lambda happened");
        output.clear();
        model.delta_int();
        assert!(model.delta_int, "delta_int happened");

        model.delta_ext(Duration::from_millis(2500), &());
        assert_eq!(
            model.delta_ext_elapsed,
            Duration::from_millis(2500),
            "Correct elapsed after delta_ext"
        );

        assert_eq!(model.ta(), Duration::from_secs(7), "ta with correct value");

        model.lambda(&mut output);
        assert_eq!(output.get_values(), &[99], "lambda happened (2nd call)");
        output.clear();
        model.delta_int = false;
        model.delta_conf(&());
        assert!(model.delta_int, "delta_conf default calls delta_int");
        assert_eq!(
            model.delta_ext_elapsed,
            Duration::from_secs(0),
            "delta_conf default calls delta_ext(elapsed=0)"
        );

        model.stop();
        assert!(model.stop, "stop happened");
    }

    #[test]
    fn ref_mut_delegates_all_methods() {
        let mut raw = CallTracker {
            start: false,
            stop: false,
            delta_int: false,
            delta_ext_elapsed: Duration::MAX,
            ta_val: Duration::from_secs(7),
        };

        let mut output = Port::<usize, 1>::new();

        <&mut CallTracker as Atomic>::start(&mut &mut raw);
        assert!(raw.start, "start delegates through &mut T blanket");

        <&mut CallTracker as Atomic>::lambda(&&mut raw, &mut output);
        assert_eq!(
            output.get_values(),
            &[99],
            "lambda delegates through &mut T blanket"
        );
        output.clear();
        <&mut CallTracker as Atomic>::delta_int(&mut &mut raw);
        assert!(raw.delta_int, "delta_int delegates through &mut T blanket");

        <&mut CallTracker as Atomic>::delta_ext(&mut &mut raw, Duration::from_millis(2500), &());
        assert_eq!(
            raw.delta_ext_elapsed,
            Duration::from_millis(2500),
            "delta_ext delegates through &mut T blanket with correct elapsed"
        );

        assert_eq!(
            <&mut CallTracker as Atomic>::ta(&&mut raw),
            Duration::from_secs(7),
            "ta delegates through &mut T blanket"
        );

        <&mut CallTracker as Atomic>::lambda(&&mut raw, &mut output);
        assert_eq!(
            output.get_values(),
            &[99],
            "lambda happened on second call through &mut T blanket"
        );
        output.clear();
        raw.delta_int = false;
        <&mut CallTracker as Atomic>::delta_conf(&mut &mut raw, &());
        assert!(
            raw.delta_int,
            "delta_conf default calls delta_int through &mut T blanket"
        );
        assert_eq!(
            raw.delta_ext_elapsed,
            Duration::from_secs(0),
            "delta_conf default calls delta_ext(elapsed=0) through &mut T blanket"
        );

        <&mut CallTracker as Atomic>::stop(&mut &mut raw);
        assert!(raw.stop, "stop delegates through &mut T blanket");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn box_delegates_all_methods() {
        let mut raw = alloc::boxed::Box::new(CallTracker {
            start: false,
            stop: false,
            delta_int: false,
            delta_ext_elapsed: Duration::MAX,
            ta_val: Duration::from_secs(7),
        });

        let mut output = Port::<usize, 1>::new();

        <alloc::boxed::Box<CallTracker> as Atomic>::start(&mut raw);
        assert!(raw.start, "start delegates through Box<T>");

        <alloc::boxed::Box<CallTracker> as Atomic>::lambda(&raw, &mut output);
        assert_eq!(
            output.get_values(),
            &[99],
            "lambda delegates through Box<T> blanket"
        );
        output.clear();
        <alloc::boxed::Box<CallTracker> as Atomic>::delta_int(&mut raw);
        assert!(raw.delta_int, "delta_int delegates through Box<T>");

        <alloc::boxed::Box<CallTracker> as Atomic>::delta_ext(
            &mut raw,
            Duration::from_millis(2500),
            &(),
        );
        assert_eq!(
            raw.delta_ext_elapsed,
            Duration::from_millis(2500),
            "delta_ext delegates through Box<T> with correct elapsed"
        );

        assert_eq!(
            <alloc::boxed::Box<CallTracker> as Atomic>::ta(&raw),
            Duration::from_secs(7),
            "ta delegates through Box<T>"
        );

        <alloc::boxed::Box<CallTracker> as Atomic>::lambda(&raw, &mut output);
        assert_eq!(
            output.get_values(),
            &[99],
            "lambda delegates through Box<T> blanket"
        );
        output.clear();
        raw.delta_int = false;
        <alloc::boxed::Box<CallTracker> as Atomic>::delta_conf(&mut raw, &());
        assert!(
            raw.delta_int,
            "delta_conf default calls delta_int through Box<T>"
        );
        assert_eq!(
            raw.delta_ext_elapsed,
            Duration::from_secs(0),
            "delta_conf default calls delta_ext(elapsed=0) through Box<T> delegation"
        );

        <alloc::boxed::Box<CallTracker> as Atomic>::stop(&mut raw);
        assert!(raw.stop, "stop delegates through Box<T>");
    }
}
