// use xdevs::traits::{AbstractSimulator, Component};

//Inicio modelo atómico sencillo que mete datos en el puerto de entrada del modelo LI
#[xdevs::atomic]
pub struct Generator {
    #[output]
    out_job: xdevs::port::Port<usize, 1>,
    #[state]
    sigma: f64,
    count: usize,
}

impl xdevs::Atomic for Generator {
    fn delta_int(state: &mut Self::State) {
        state.sigma = f64::INFINITY;
    }

    fn lambda(state: &Self::State, output: &mut Self::Output) {
        output.out_job.add_value(state.count).unwrap();
    }

    fn ta(state: &Self::State) -> f64 {
        state.sigma
    }

    fn delta_ext(_state: &mut Self::State, _elapsed: f64, _input: &Self::Input) {}
}

impl Generator {
    pub fn new(val_count: usize) -> Self {
        Self::build(0.0, val_count)
    }
}
//Fin modelo atómico sencillo que mete datos en el puerto de entrada del modelo LI

//Inicio del modelo atómico que va dentro de los acoplados en un array de atómicos
#[xdevs::atomic]
pub struct Atom {
    #[input]
    input_port: xdevs::port::Port<usize, 1>,
    #[output]
    output_port: xdevs::port::Port<usize, 1>,
    #[state]
    sigma: f64,
    n_internals: usize,
    n_externals: usize, //debería ser igual que n_internals
    n_events: usize,    //número de eventos que llegan al puerto acumulador
}

impl xdevs::Atomic for Atom {
    fn delta_int(state: &mut Self::State) {
        state.sigma = f64::INFINITY;
        state.n_internals += 1;
    }

    fn lambda(state: &Self::State, output: &mut Self::Output) {
        output.output_port.add_value(state.n_events).unwrap();
    }

    fn ta(state: &Self::State) -> f64 {
        state.sigma
    }

    fn delta_ext(state: &mut Self::State, _elapsed: f64, input: &Self::Input) {
        // state.sigma -= elapsed;
        state.sigma = 0.0;
        state.n_externals += 1;
        state.n_events += input.input_port.get_values().len();
    }
}

impl Atom {
    pub fn new() -> Self {
        Self::build(f64::INFINITY, 0, 0, 0)
    }

    pub fn get_n_internals(&self) -> usize {
        self.state.n_internals
    }

    pub fn get_n_externals(&self) -> usize {
        self.state.n_externals
    }

    pub fn get_n_events(&self) -> usize {
        self.state.n_events
    }

    pub fn get_n_atomics(&self) -> usize {
        1
    }
}
//Fin del modelo atómico que va dentro de los acoplados en un array de atómicos

//Inicio del modelo acoplado CoupAtom que contiene un único atómico
// #[xdevs::coupled2]
// pub struct CoupAtom {
//     #[input]
//     input_port: xdevs::port::Port<usize, 1>,
//     #[output]
//     output_port: xdevs::port::Port<usize, 1>,
//     #[components]
//     coup_atomic: Atom,
// }

// Recursive expansion of coupled2 macro
// ======================================

#[derive(Debug, Default)]
pub struct CoupInputPort {
    pub input_port: xdevs::port::Port<usize, 1>,
}
impl CoupInputPort {
    #[inline]
    pub const fn new() -> Self {
        Self {
            input_port: xdevs::port::Port::new(),
        }
    }
}
unsafe impl xdevs::traits::Bag for CoupInputPort {
    #[inline]
    fn is_empty(&self) -> bool {
        true && self.input_port.is_empty()
    }
    #[inline]
    fn clear(&mut self) {
        self.input_port.clear();
    }
}
#[derive(Debug, Default)]
pub struct CoupOutputPort {
    pub output_port: xdevs::port::Port<usize, 1>,
}
impl CoupOutputPort {
    #[inline]
    pub const fn new() -> Self {
        Self {
            output_port: xdevs::port::Port::new(),
        }
    }
}
unsafe impl xdevs::traits::Bag for CoupOutputPort {
    #[inline]
    fn is_empty(&self) -> bool {
        true && self.output_port.is_empty()
    }
    #[inline]
    fn clear(&mut self) {
        self.output_port.clear();
    }
}
pub struct CoupAtomComponents {
    coup_atomic: Atom,
}
impl CoupAtomComponents {
    #[inline]
    pub fn new(coup_atomic: Atom) -> Self {
        Self { coup_atomic }
    }
}
#[doc = r" Wrapper struct holding mutable references to all inner components' inputs."]
pub struct CoupAtomComponentsInput<'__xdevs_inner> {
    pub coup_atomic: <Atom as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
}
#[doc = r" Wrapper struct holding references to all inner components' outputs."]
pub struct CoupAtomComponentsOutput<'__xdevs_inner> {
    pub coup_atomic: <Atom as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
}
pub struct CoupAtom {
    pub input: CoupInputPort,
    pub output: CoupOutputPort,
    pub t_last: f64,
    pub t_next: f64,
    pub components: CoupAtomComponents,
}
impl CoupAtom {
    #[inline]
    pub fn build(coup_atomic: Atom) -> Self {
        Self {
            input: CoupInputPort::new(),
            output: CoupOutputPort::new(),
            t_last: 0.0,
            t_next: f64::INFINITY,
            components: CoupAtomComponents::new(coup_atomic),
        }
    }
}
unsafe impl xdevs::traits::Component for CoupAtom {
    type Input = CoupInputPort;
    type Output = CoupOutputPort;
    type InputRef<'__xdevs_ports>
        = &'__xdevs_ports mut CoupInputPort
    where
        Self: '__xdevs_ports;
    type OutputRef<'__xdevs_ports>
        = &'__xdevs_ports CoupOutputPort
    where
        Self: '__xdevs_ports;
    #[inline]
    fn get_t_last(&self) -> f64 {
        self.t_last
    }
    #[inline]
    fn set_t_last(&mut self, t_last: f64) {
        self.t_last = t_last;
    }
    #[inline]
    fn get_t_next(&self) -> f64 {
        self.t_next
    }
    #[inline]
    fn set_t_next(&mut self, t_next: f64) {
        self.t_next = t_next;
    }
    #[inline]
    fn get_input(&self) -> &Self::Input {
        &self.input
    }
    #[inline]
    fn get_input_mut(&mut self) -> &mut Self::Input {
        &mut self.input
    }
    #[inline]
    fn get_output(&self) -> &Self::Output {
        &self.output
    }
    #[inline]
    fn get_output_mut(&mut self) -> &mut Self::Output {
        &mut self.output
    }
    #[inline]
    fn get_ports(&mut self) -> (Self::InputRef<'_>, Self::OutputRef<'_>) {
        (&mut self.input, &self.output)
    }
    #[inline]
    fn get_out_ports(&self) -> Self::OutputRef<'_> {
        &self.output
    }
}
unsafe impl xdevs::traits::PartialCoupled for CoupAtom {
    type ComponentsInput<'__xdevs_inner>
        = CoupAtomComponentsInput<'__xdevs_inner>
    where
        Self: '__xdevs_inner;
    type ComponentsOutput<'__xdevs_inner>
        = CoupAtomComponentsOutput<'__xdevs_inner>
    where
        Self: '__xdevs_inner;
}
unsafe impl xdevs::traits::AbstractSimulator for CoupAtom {
    #[inline]
    fn start(&mut self, t_start: f64) -> f64 {
        xdevs::traits::Component::set_t_last(self, t_start);
        let mut t_next = f64::INFINITY;
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::start(&mut self.components.coup_atomic, t_start),
        );
        xdevs::traits::Component::set_t_next(self, t_next);
        t_next
    }
    #[inline]
    fn stop(&mut self, t_stop: f64) {
        xdevs::traits::AbstractSimulator::stop(&mut self.components.coup_atomic, t_stop);
        xdevs::traits::Component::set_t_last(self, t_stop);
        xdevs::traits::Component::set_t_next(self, f64::INFINITY);
    }
    #[inline]
    fn lambda(&mut self, t: f64) {
        if t >= xdevs::traits::Component::get_t_next(self) {
            xdevs::traits::AbstractSimulator::lambda(&mut self.components.coup_atomic, t);
            let coup_atomic_output =
                xdevs::traits::Component::get_out_ports(&self.components.coup_atomic);
            let component_outputs: CoupAtomComponentsOutput<'_> = CoupAtomComponentsOutput {
                coup_atomic: coup_atomic_output,
            };
            <Self as xdevs::Coupled>::eoc(&component_outputs, &mut self.output);
        }
    }
    #[inline]
    fn delta(&mut self, t: f64) -> f64 {
        {
            let (coup_atomic_input, coup_atomic_output) =
                xdevs::traits::Component::get_ports(&mut self.components.coup_atomic);
            let component_outputs: CoupAtomComponentsOutput<'_> = CoupAtomComponentsOutput {
                coup_atomic: coup_atomic_output,
            };
            let mut component_inputs: CoupAtomComponentsInput<'_> = CoupAtomComponentsInput {
                coup_atomic: coup_atomic_input,
            };
            <Self as xdevs::Coupled>::eic(&self.input, &mut component_inputs);
            <Self as xdevs::Coupled>::ic(&component_outputs, &mut component_inputs);
        }
        let mut t_next = f64::INFINITY;
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::delta(&mut self.components.coup_atomic, t),
        );
        xdevs::traits::Component::clear_output(self);
        xdevs::traits::Component::clear_input(self);
        xdevs::traits::Component::set_t_last(self, t);
        xdevs::traits::Component::set_t_next(self, t_next);
        t_next
    }
}

impl CoupAtom {
    pub fn new() -> Self {
        Self::build(Atom::new())
    }

    pub fn get_n_internals(&self) -> usize {
        self.components.coup_atomic.get_n_internals()
    }

    pub fn get_n_externals(&self) -> usize {
        self.components.coup_atomic.get_n_externals()
    }

    pub fn get_n_events(&self) -> usize {
        self.components.coup_atomic.get_n_events()
    }

    pub fn get_n_atomics(&self) -> usize {
        self.components.coup_atomic.get_n_atomics()
    }
}

impl xdevs::Coupled for CoupAtom {
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput<'_>) {
        from.input_port.couple(&mut to.coup_atomic.input_port);
    }
    fn eoc(from: &Self::ComponentsOutput<'_>, to: &mut Self::Output) {
        from.coup_atomic.output_port.couple(&mut to.output_port);
    }
}
//Fin modelo acoplado CoupAtom que contiene un único atómico

//Inicio atómico con puerto de entrada de tamaño 2 y 1 de salida
#[xdevs::atomic]
pub struct AtomInputSize2 {
    #[input]
    input_port: xdevs::port::Port<usize, 2>, //un único input_port de tamaño 2
    #[output]
    output_port: xdevs::port::Port<usize, 1>,
    #[state]
    sigma: f64,
    n_internals: usize,
    n_externals: usize,
    n_events: usize,
}

impl xdevs::Atomic for AtomInputSize2 {
    fn delta_int(state: &mut Self::State) {
        state.sigma = f64::INFINITY;
        state.n_internals += 1;
    }

    fn lambda(state: &Self::State, output: &mut Self::Output) {
        output.output_port.add_value(state.n_events).unwrap();
    }

    fn ta(state: &Self::State) -> f64 {
        state.sigma
    }

    fn delta_ext(state: &mut Self::State, _elapsed: f64, input: &Self::Input) {
        state.sigma = 0.0;
        state.n_externals += 1;
        state.n_events += input.input_port.get_values().len();
    }
}

impl AtomInputSize2 {
    pub fn new() -> Self {
        Self::build(f64::INFINITY, 0, 0, 0)
    }

    pub fn get_n_internals(&self) -> usize {
        self.state.n_internals
    }

    pub fn get_n_externals(&self) -> usize {
        self.state.n_externals
    }

    pub fn get_n_events(&self) -> usize {
        self.state.n_events
    }

    pub fn get_n_atomics(&self) -> usize {
        1
    }
}
//Fin atómico con puerto de entrada de tamaño 2 y 1 de salida

//Inicio atómico con 2 puertos de entrada  y 2 de salida
#[xdevs::atomic]
pub struct Atom2Inputs2Outputs {
    #[input]
    input_port_1: xdevs::port::Port<usize, 1>,
    input_port_2: xdevs::port::Port<usize, 1>,
    #[output]
    output_port_1: xdevs::port::Port<usize, 1>,
    output_port_2: xdevs::port::Port<usize, 1>,
    #[state]
    sigma: f64,
    n_internals: usize,
    n_externals: usize,
    n_events: usize,
}

impl xdevs::Atomic for Atom2Inputs2Outputs {
    fn delta_int(state: &mut Self::State) {
        state.sigma = f64::INFINITY;
        state.n_internals += 1;
    }

    fn lambda(state: &Self::State, output: &mut Self::Output) {
        output.output_port_1.add_value(state.n_events).unwrap();
        output.output_port_2.add_value(state.n_events).unwrap();
    }

    fn ta(state: &Self::State) -> f64 {
        state.sigma
    }

    fn delta_ext(state: &mut Self::State, _elapsed: f64, input: &Self::Input) {
        state.sigma = 0.0;
        state.n_externals += 1;
        state.n_events +=
            input.input_port_1.get_values().len() + input.input_port_2.get_values().len();
    }
}

impl Atom2Inputs2Outputs {
    pub fn new() -> Self {
        Self::build(f64::INFINITY, 0, 0, 0)
    }

    pub fn get_n_internals(&self) -> usize {
        self.state.n_internals
    }

    pub fn get_n_externals(&self) -> usize {
        self.state.n_externals
    }

    pub fn get_n_events(&self) -> usize {
        self.state.n_events
    }

    pub fn get_n_atomics(&self) -> usize {
        1
    }
}
//Fin atómico con 2 puertos de entrada y 2 de salida
