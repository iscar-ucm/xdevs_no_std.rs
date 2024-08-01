extern crate std;
use std::boxed::Box;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};
use std::vec::Vec;

/// Closure for RT simulation on targets with `std`.
/// It sleeps until the next state transition.
/// It is an implementation of the `wait_until` closure for the `Simulator::simulate_rt` method.
pub fn sleep<T: crate::traits::Bag>(
    t_start: f64,
    time_scale: f64,
    max_jitter: Option<std::time::Duration>,
) -> impl FnMut(f64, &mut T) -> f64 {
    wait_event(t_start, time_scale, max_jitter, |waiting_period, _| {
        std::thread::sleep(waiting_period)
    })
}

/// It computes the next wall-clock time corresponding to the next state transition of the model.
///
/// It is an implementation of the `wait_until` closure for the `Simulator::simulate_rt` method.
/// In this implementation, an input handler function waits for external events without exceeding the time for the next internal event.
/// Finally, it checks that the wall-clock drift does not exceed the maximum jitter allowed (if any) and panics if it does.
///
///  # Arguments
///
///  * `t_start` - The virtual time at the beginning of the simulation.
///  * `time_scale` - The time scale factor between virtual and wall-clock time.
///  * `max_jitter` - The maximum allowed jitter duration. If `None`, no jitter check is performed.
///  * `input_handler` - The function to handle incoming external events. This function expects two arguments:
///    - `duration: [Duration]` - Maximum duration of the time interval to wait for external events.
///      The input handler function may return earlier if an input event is received.
///      Note, however, that it must **NOT** return after, as it would result in an incorrect real-time implementation.
///    - `input_ports: &mut T` - Mutable reference to the input ports of the top-most model under simulation.
///    
///  # Returns
///
///  A closure that takes the next virtual time and a mutable reference to the bag and returns the next virtual time.
///
/// # Example
///
/// ```ignore
/// xdevs::simulator::std::wait_event(0., 1., Some(Duration::from_millis(50)), some_input_handler);
/// ```

pub fn wait_event<T: crate::traits::Bag>(
    t_start: f64,
    time_scale: f64,
    max_jitter: Option<Duration>,
    mut input_handler: impl FnMut(Duration, &mut T),
) -> impl FnMut(f64, &mut T) -> f64 {
    let mut last_vt = t_start;
    let mut last_rt = SystemTime::now();
    let start_rt = last_rt;

    move |t_next, binput: &mut T| -> f64 {
        assert!(t_next >= last_vt);

        let next_rt = last_rt + Duration::from_secs_f64((t_next - last_vt) * time_scale);

        if let Ok(duration) = next_rt.duration_since(SystemTime::now()) {
            input_handler(duration, binput);
        }

        let t = SystemTime::now();

        last_vt = match t.duration_since(next_rt) {
            Ok(duration) => {
                // t >= next_rt, check for the jitter
                if let Some(max_jitter) = max_jitter {
                    if duration > max_jitter {
                        panic!("[WE]>> Jitter too high: {:?}", duration);
                    }
                }
                last_rt = next_rt;
                t_next
            }
            Err(_) => {
                // t < next_rt
                last_rt = t;
                let duration = last_rt.duration_since(start_rt).unwrap();
                duration.as_secs_f64() / time_scale
            }
        };

        last_vt
    }
}

// OutputHandlers implementation

type OutputHandler<T> = Box<dyn FnMut(&T)>;

/// A struct that represents a multiple output handler.
///
/// It contains a vector of `OutputHandler<T>` instances.
pub struct MultipleOutputHandler<T> {
    ohs: Vec<OutputHandler<T>>,
}

impl<T> Default for MultipleOutputHandler<T> {
    fn default() -> Self {
        Self { ohs: Vec::new() }
    }
}

/// A struct representing a multiple output handler for a generic type `T`.
impl<T: crate::traits::Bag> MultipleOutputHandler<T> {
    /// Creates a new instance of `MultipleOutputHandler`.
    ///
    /// # Returns
    ///
    /// A new instance of `MultipleOutputHandler`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an output handler to the `MultipleOutputHandler`.
    ///
    /// # Arguments
    ///
    /// * `oh` - A boxed closure that takes a reference to `T` as input.
    pub fn add_output_handler(&mut self, oh: Box<dyn FnMut(&T)>) {
        self.ohs.push(oh);
    }

    /// Converts the `MultipleOutputHandler` into a closure that can be called with a reference to `T`.
    ///
    /// # Returns
    ///
    /// A closure that takes a reference to `T` as input and calls all the output handlers with that reference.
    pub fn into_handler(mut self) -> impl FnMut(&T) {
        move |t: &T| {
            self.ohs.iter_mut().for_each(|oh| oh(t));
        }
    }
}

/* pub trait Event {
    type EventInfo;
    fn default() -> Self;
} */

/// A trait for handling a generic event.
///
/// The trait is used to define the default behaviour that events must implement.
/// All events must implement this trait since there can be errors in the simulation and a default event must be returned.
///
/// # Examples
///
/// ```
/// pub struct AnyEvent {
///     pub some_type_of_data: (String, String),
/// }
///
/// impl Event for AnyEvent {
///     fn default() -> Self {
///         AnyEvent {
///             some_type_of_data: ("".to_string(), "".to_string()),
///         }
///     }
/// }
/// ```
pub trait Event {
    /// Creates a default instance of the event.
    ///
    /// # Examples
    ///
    /// ```
    /// struct MyEvent;
    ///
    /// impl Event for MyEvent {
    ///
    ///     fn default() -> Self {
    ///         MyEvent
    ///     }
    /// }
    ///
    /// let event = MyEvent::default();
    /// ```
    fn default() -> Self;
}

/// A framework for adding Input Handlers and reducing the complexity of their use.
///
/// This struct manages input handlers by spawning threads for each handler and managing the communication
/// between these threads and the main application. It uses a multi-producer, single-consumer (mpsc) channel
/// to send and receive events.
///
/// # Type Parameters
/// * `T` - It represents the input bag of the model under simulation. Usually it is named `ModelNameInput`.
/// * `F` - The function type used to inject events into the model. This function is implementaion-specific and it takes a mutable reference to T and event U as input.
/// * `U` - It is the event type handled by the mpsc channel. It must implement the `Default` trait.
///
/// # Fields
/// * `phantom` - A phantom data marker to hold the generic type `T`.
/// * `tx` - The sender side of the mpsc channel used to send events.
/// * `rx` - The receiver side of the mpsc channel used to receive events.
/// * `thread_handles` - A vector holding the handles of the spawned threads.
/// * `window_time` - An optional duration representing the window time for collecting extra events.
/// * `inject_event` - A function used to inject events into the model.
pub struct InputHandlersManager<T, F, U>
where
    F: FnMut(&mut T, U) + 'static,
{
    phantom: std::marker::PhantomData<T>,
    tx: std::sync::mpsc::Sender<U>,
    rx: std::sync::mpsc::Receiver<U>,
    thread_handles: Vec<std::thread::JoinHandle<()>>,
    window_time: Option<Duration>,
    inject_event: F,
}

impl<T, F, U> InputHandlersManager<T, F, U>
where
    T: crate::traits::Bag,
    F: FnMut(&mut T, U),
    U: core::default::Default + Send + 'static,
{
    /// Creates a new `InputHandlersManager`.
    ///
    /// # Parameters
    /// * `window_time` - An optional duration representing the window time for collecting evetns consecutive in a time window.
    /// * `inject_event` - An implementation-specific function used to inject events into the model. It takes a mutable reference to `T` and an event `U` as input.
    ///
    /// # Returns
    /// A new instance of `InputHandlersManager`.
    pub fn new(window_time: Option<Duration>, inject_event: F) -> Self {
        let (tx, rx) = mpsc::channel::<U>();
        Self {
            phantom: std::marker::PhantomData,
            tx,
            rx,
            thread_handles: Vec::new(),
            window_time,
            inject_event,
        }
    }

    /// Adds a new input handler and spawns a thread for it.
    ///
    /// # Parameters
    /// * `ih` - A closure representing the input handler that will be run in a new thread. It takes a sender side of the mpsc channel as input.
    pub fn add_input_handler(&mut self, mut ih: impl FnMut(mpsc::Sender<U>) + Send + 'static) {
        let tx_cloned = self.tx.clone();
        self.thread_handles
            .push(thread::spawn(move || ih(tx_cloned)));
    }

    /// Converts the `InputHandlersManager` into a function that plays the role of a single input handler in the simulation.
    ///
    /// Now the InputHandlersManager acts as a single input handler that collects all the events from the mpsc channel.
    /// Additionally, it will inject the events into the model using the provided `inject_event` function.
    ///
    /// # Returns
    /// A closure that collects events and injects them into the model.
    pub fn into_ihandler(self) -> impl FnMut(Duration, &mut T) {
        Self::collect_events(self.rx, self.window_time, self.inject_event)
    }

    /// Collects events and injects them into the model.
    ///
    /// This function continuously collects events from the receiver of the mpsc channel within a specified duration and
    /// injects them into the model using the provided `inject_event` function.
    /// If a window time is specified, it will keep collecting events until the window time is reached.
    ///
    /// This function is used to convert the behaviour of the `InputHandlersManager` into a single input handler.
    ///
    /// # Parameters
    /// * `rx` - The receiver side of the mpsc channel used to receive events.
    /// * `window_time` - An optional duration representing the window time for collecting events.
    /// * `inject_event` - A function used to inject events into the model.
    ///
    /// # Returns
    /// A closure that collects events and injects them into the model.
    fn collect_events(
        rx: mpsc::Receiver<U>,
        window_time: Option<Duration>,
        mut inject_event: impl FnMut(&mut T, U),
    ) -> impl FnMut(Duration, &mut T) {
        move |duration, model| {
            // Time elapsed during the event collection window, initially 0
            let mut t_welapsed = Duration::from_secs(0);
            // Start time of the event collection
            let t_ini = SystemTime::now();

            // Wait for the first event within the specified duration
            let event: U = _get_event(&rx, duration);
            // Inject the received event into the model
            inject_event(model, event);

            // Total elapsed time with the first event
            let t_end = SystemTime::now();
            let t_elapsed = t_end.duration_since(t_ini).unwrap();

            if t_elapsed > duration {
                return;
            }
            // If there is a window time specified...
            if let Some(w) = window_time {
                // Determine the window duration considering the elapsed time and the total duration
                let window = core::cmp::min(w, duration - t_elapsed);

                while window > t_welapsed {
                    // Wait for the next event within the remaining window time
                    let event = _get_event(&rx, window - t_welapsed);
                    // Update the elapsed window time
                    t_welapsed = SystemTime::now().duration_since(t_end).unwrap();

                    // Inject the received event into the model
                    inject_event(model, event);
                }
            }

            /// Internal function to get the event from the receiver within the specified duration
            fn _get_event<U: core::default::Default>(rx: &mpsc::Receiver<U>, d: Duration) -> U {
                // Wait for the event within the specified duration
                rx.recv_timeout(d).unwrap_or_else(|err| {
                    // Return a default event if no message is received before the timeout
                    if err == mpsc::RecvTimeoutError::Timeout {
                        U::default()
                    } else {
                        panic!("Error receiving message: {:?}", err);
                    }
                })
            }
        }
    }
}
