use xdevs::traits::{AbstractSimulator, Component};

//Atomic model
// #[xdevs::atomic]
// pub struct Atom {
//     #[input]
//     input_port: xdevs::port::Port<bool, 1>,
//     #[output]
//     output_port: xdevs::port::Port<usize, 1>,
//     #[state]
//     sigma: f64,
//     period: f64,
//     count: usize,
// }

// Recursive expansion of atomic macro
// ====================================

#[derive(Debug)]
pub struct AtomState {
    sigma: f64,
    period: f64,
    count: usize,
}
impl AtomState {
    #[inline]
    pub fn new(sigma: f64, period: f64, count: usize) -> Self {
        Self {
            sigma,
            period,
            count,
        }
    }
}
pub struct Atom {
    pub input: ModCoupLIInput,
    pub output: ModCoupLIOutput,
    pub t_last: f64,
    pub t_next: f64,
    pub state: AtomState,
}
impl Atom {
    #[inline]
    pub fn build(sigma: f64, period: f64, count: usize) -> Self {
        Self {
            input: ModCoupLIInput::new(),
            output: ModCoupLIOutput::new(),
            t_last: 0.0,
            t_next: f64::INFINITY,
            state: AtomState::new(sigma, period, count),
        }
    }
}
unsafe impl xdevs::traits::Component for Atom {
    type Input = ModCoupLIInput;
    type Output = ModCoupLIOutput;
    type InputRef<'__xdevs_ports>
        = &'__xdevs_ports mut Self::Input
    where
        Self: '__xdevs_ports;
    type OutputRef<'__xdevs_ports>
        = &'__xdevs_ports Self::Output
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
unsafe impl xdevs::traits::PartialAtomic for Atom {
    type State = AtomState;
}
unsafe impl xdevs::traits::AbstractSimulator for Atom {
    #[inline]
    fn start(&mut self, t_start: f64) -> f64 {
        xdevs::traits::Component::set_t_last(self, t_start);
        <Self as xdevs::Atomic>::start(&mut self.state);
        let t_next = t_start + <Self as xdevs::Atomic>::ta(&self.state);
        xdevs::traits::Component::set_t_next(self, t_next);
        t_next
    }
    #[inline]
    fn stop(&mut self, t_stop: f64) {
        <Self as xdevs::Atomic>::stop(&mut self.state);
        xdevs::traits::Component::set_t_last(self, t_stop);
        xdevs::traits::Component::set_t_next(self, f64::INFINITY);
    }
    #[inline]
    fn lambda(&mut self, t: f64) {
        if t >= xdevs::traits::Component::get_t_next(self) {
            <Self as xdevs::Atomic>::lambda(&self.state, &mut self.output);
        }
    }
    #[inline]
    fn delta(&mut self, t: f64) -> f64 {
        let mut t_next = xdevs::traits::Component::get_t_next(self);
        if !xdevs::traits::Bag::is_empty(&self.input) {
            if t >= t_next {
                <Self as xdevs::Atomic>::delta_conf(&mut self.state, &self.input);
            } else {
                let e = t - xdevs::traits::Component::get_t_last(self);
                <Self as xdevs::Atomic>::delta_ext(&mut self.state, e, &self.input);
            }
            xdevs::traits::Component::clear_input(self);
        } else if t >= t_next {
            <Self as xdevs::Atomic>::delta_int(&mut self.state);
        } else {
            return t_next;
        }
        xdevs::traits::Component::clear_output(self);
        t_next = t + <Self as xdevs::Atomic>::ta(&self.state);
        xdevs::traits::Component::set_t_last(self, t);
        xdevs::traits::Component::set_t_next(self, t_next);
        t_next
    }
}

impl xdevs::Atomic for Atom {
    fn delta_int(state: &mut Self::State) {
        state.count += 1;
        state.sigma = state.period;
    }

    fn lambda(state: &Self::State, output: &mut Self::Output) {
        output.output_port.add_value(state.count).unwrap();
    }

    fn ta(state: &Self::State) -> f64 {
        state.sigma
    }

    fn delta_ext(state: &mut Self::State, elapsed: f64, input: &Self::Input) {
        state.sigma -= elapsed;
        if let Some(&stop) = input.input_port.get_values().last() {
            if stop {
                state.sigma = f64::INFINITY;
            }
        }
    }
}

impl Atom {
    pub fn new(period: f64) -> Self {
        Self::build(0.0, period, 0)
    }
}
//Fin atomic model

/*ENUM: hay 2 opciones
- Opción 1: acoplado con un único atomic
- Opción 2: acoplado con un atomic y un acoplado, y el acoplado a su vez con un atomic y un acoplado, etc. (estructura recursiva)
*/

//Inicio modelo acoplado CoupAtom que contiene un único atómico
pub struct CoupAtom {
    coup_atomic: Atom,
}

impl CoupAtom {
    pub fn new(period: f64) -> Self {
        Self {
            coup_atomic: Atom::new(period),
        }
    }
}
//Fin modelo acoplado CoupAtom que contiene un único atómico

//Inicio modelo acoplado Coup
// pub enum Coup<const W: usize, const P: usize> {
//     CoupD(Atom), //en vez de Atom, Coup (como no es recursivo no hace falta pasarlo por referencia)
//     RestoCoup(Box<Self>), //RestoCoup(Box<Coup<W, P>>),
// }

pub enum Coup<const W: usize, const P: usize> {
    CoupD(CoupAtom),
    RestoCoup(Box<Coup<W, P>>),
}

//REVISAR
impl<const W: usize, const P: usize> Coup<W, P> {
    fn new(&self, period: f64) -> Self {
        match self {
            Coup::CoupD(coup_atom) => Coup::CoupD(CoupAtom::new(period)),
            Coup::RestoCoup(coup) => Coup::RestoCoup(Box::new(Coup::new(coup, period))),
        }
    }
}
//Fin modelo acoplado Coup

//"Interfaz" de los inputs y outputs del modelo acoplado, para poder usarlo con el simulador
// pub fn interfaz_puertos(
//     atom_input: &atom::ModCoupLIInput,
//     atom_output: &atom::ModCoupLIOutput,
// ) -> (ModCoupLIInput, ModCoupLIOutput) {
//     let mut atom_modcoup_input = ModCoupLIInput::new();
//     let mut atom_modcoup_output = ModCoupLIOutput::new();
//     //Conecto los puertos del atomic con los del acoplado
//     if let Some(&value) = atom_input.input_port.get_values().last() {
//         atom_modcoup_input.input_port.add_value(value).unwrap()
//     }
//     if let Some(&value) = atom_output.output_port.get_values().last() {
//         atom_modcoup_output.output_port.add_value(value).unwrap()
//     }
//     (atom_modcoup_input, atom_modcoup_output)
// }

// pub fn interfaz_puerto_input(atom: &atom::ModCoupLIInput) -> (ModCoupLIInput) {
//     let mut atom_modcoup_input = ModCoupLIInput::new();
//     if let Some(&value) = atom.input_port.get_values().last() {
//         atom_modcoup_input.input_port.add_value(value).unwrap()
//     }
//     atom_modcoup_input
// }

// pub fn interfaz_puerto_output(atom: &atom::ModCoupLIOutput) -> (ModCoupLIOutput) {
//     let mut atom_modcoup_output = ModCoupLIOutput::new();
//     if let Some(&value) = atom.output_port.get_values().last() {
//         atom_modcoup_output.output_port.add_value(value).unwrap()
//     }
//     atom_modcoup_output
// }

//Inicio modelo LI CoupModLI
// #[xdevs::coupled2]
// pub struct ModCoupLI<const W: usize, const P: usize> {
//     #[input]
//     input_port: xdevs::port::Port<bool, 1>,
//     #[output]
//     output_port: xdevs::port::Port<usize, 1>,
//     #[components]
//     //constante con la width y otra que sea la width-1 para los genéricos
//     component_li: Box<Coup<W, P>>,
// }
// Recursive expansion of coupled2 macro
// ======================================

#[derive(Debug, Default)]
pub struct ModCoupLIInput {
    pub input_port: xdevs::port::Port<bool, 1>,
}
impl ModCoupLIInput {
    #[inline]
    pub const fn new() -> Self {
        Self {
            input_port: xdevs::port::Port::new(),
        }
    }
}
unsafe impl xdevs::traits::Bag for ModCoupLIInput {
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
pub struct ModCoupLIOutput {
    pub output_port: xdevs::port::Port<usize, 1>,
}
impl ModCoupLIOutput {
    #[inline]
    pub const fn new() -> Self {
        Self {
            output_port: xdevs::port::Port::new(),
        }
    }
}
unsafe impl xdevs::traits::Bag for ModCoupLIOutput {
    #[inline]
    fn is_empty(&self) -> bool {
        true && self.output_port.is_empty()
    }
    #[inline]
    fn clear(&mut self) {
        self.output_port.clear();
    }
}
pub struct ModCoupLIComponents<const W: usize, const P: usize> {
    component_li: Box<Coup<W, P>>,
}
impl<const W: usize, const P: usize> ModCoupLIComponents<W, P> {
    #[inline]
    pub fn new(component_li: Box<Coup<W, P>>) -> Self {
        Self { component_li }
    }
}
#[doc = r" Wrapper struct holding mutable references to all inner components' inputs."]
pub struct ModCoupLIComponentsInput<'__xdevs_inner, const W: usize, const P: usize> {
    pub component_li: <Box<Coup<W, P>> as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
}
#[doc = r" Wrapper struct holding references to all inner components' outputs."]
pub struct ModCoupLIComponentsOutput<'__xdevs_inner, const W: usize, const P: usize> {
    pub component_li: <Box<Coup<W, P>> as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
}
pub struct ModCoupLI<const W: usize, const P: usize> {
    pub input: ModCoupLIInput,
    pub output: ModCoupLIOutput,
    pub t_last: f64,
    pub t_next: f64,
    pub components: ModCoupLIComponents<W, P>,
}
impl<const W: usize, const P: usize> ModCoupLI<W, P> {
    #[inline]
    pub fn build(component_li: Box<Coup<W, P>>) -> Self {
        Self {
            input: ModCoupLIInput::new(),
            output: ModCoupLIOutput::new(),
            t_last: 0.0,
            t_next: f64::INFINITY,
            components: ModCoupLIComponents::new(component_li),
        }
    }
}
unsafe impl<const W: usize, const P: usize> xdevs::traits::Component for ModCoupLI<W, P> {
    type Input = ModCoupLIInput;
    type Output = ModCoupLIOutput;
    type InputRef<'__xdevs_ports>
        = &'__xdevs_ports mut ModCoupLIInput
    where
        Self: '__xdevs_ports;
    type OutputRef<'__xdevs_ports>
        = &'__xdevs_ports ModCoupLIOutput
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
unsafe impl<const W: usize, const P: usize> xdevs::traits::PartialCoupled for ModCoupLI<W, P> {
    type ComponentsInput<'__xdevs_inner>
        = ModCoupLIComponentsInput<'__xdevs_inner, W, P>
    where
        Self: '__xdevs_inner;
    type ComponentsOutput<'__xdevs_inner>
        = ModCoupLIComponentsOutput<'__xdevs_inner, W, P>
    where
        Self: '__xdevs_inner;
}
unsafe impl<const W: usize, const P: usize> xdevs::traits::AbstractSimulator for ModCoupLI<W, P> {
    #[inline]
    fn start(&mut self, t_start: f64) -> f64 {
        xdevs::traits::Component::set_t_last(self, t_start);
        let mut t_next = f64::INFINITY;
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::start(&mut self.components.component_li, t_start),
        );
        xdevs::traits::Component::set_t_next(self, t_next);
        t_next
    }
    #[inline]
    fn stop(&mut self, t_stop: f64) {
        xdevs::traits::AbstractSimulator::stop(&mut self.components.component_li, t_stop);
        xdevs::traits::Component::set_t_last(self, t_stop);
        xdevs::traits::Component::set_t_next(self, f64::INFINITY);
    }
    #[inline]
    fn lambda(&mut self, t: f64) {
        if t >= xdevs::traits::Component::get_t_next(self) {
            xdevs::traits::AbstractSimulator::lambda(&mut self.components.component_li, t);
            let component_li_output =
                xdevs::traits::Component::get_out_ports(&self.components.component_li);
            let component_outputs: ModCoupLIComponentsOutput<'_, W, P> =
                ModCoupLIComponentsOutput {
                    component_li: component_li_output,
                };
            <Self as xdevs::Coupled>::eoc(&component_outputs, &mut self.output);
        }
    }
    #[inline]
    fn delta(&mut self, t: f64) -> f64 {
        {
            let (component_li_input, component_li_output) =
                xdevs::traits::Component::get_ports(&mut self.components.component_li);
            let component_outputs: ModCoupLIComponentsOutput<'_, W, P> =
                ModCoupLIComponentsOutput {
                    component_li: component_li_output,
                };
            let mut component_inputs: ModCoupLIComponentsInput<'_, W, P> =
                ModCoupLIComponentsInput {
                    component_li: component_li_input,
                };
            <Self as xdevs::Coupled>::eic(&self.input, &mut component_inputs);
            <Self as xdevs::Coupled>::ic(&component_outputs, &mut component_inputs);
        }
        let mut t_next = f64::INFINITY;
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::delta(&mut self.components.component_li, t),
        );
        xdevs::traits::Component::clear_output(self);
        xdevs::traits::Component::clear_input(self);
        xdevs::traits::Component::set_t_last(self, t);
        xdevs::traits::Component::set_t_next(self, t_next);
        t_next
    }
}

//Fin modelo LI CoupModLI

//Implementación manual de AbstracSimulator para CoupAtom
unsafe impl AbstractSimulator for CoupAtom {
    fn start(&mut self, t_start: f64) -> f64 {
        self.coup_atomic.start(t_start)
    }

    fn stop(&mut self, t_stop: f64) {
        self.coup_atomic.stop(t_stop)
    }

    fn lambda(&mut self, t: f64) {
        self.coup_atomic.lambda(t)
    }

    fn delta(&mut self, t: f64) -> f64 {
        self.coup_atomic.delta(t)
    }
}
//Implementación manual de AbstracSimulator para Coup (la macro no lo implementa)
unsafe impl<const W: usize, const P: usize> AbstractSimulator for Coup<W, P> {
    fn start(&mut self, t_start: f64) -> f64 {
        match self {
            Coup::CoupD(d) => d.start(t_start),
            Coup::RestoCoup(r) => r.start(t_start),
        }
    }

    fn stop(&mut self, t_stop: f64) {
        match self {
            Coup::CoupD(d) => d.stop(t_stop),
            Coup::RestoCoup(r) => r.stop(t_stop),
        }
    }

    fn lambda(&mut self, t: f64) {
        match self {
            Coup::CoupD(d) => d.lambda(t),
            Coup::RestoCoup(r) => r.lambda(t),
        }
    }

    fn delta(&mut self, t: f64) -> f64 {
        match self {
            Coup::CoupD(d) => d.delta(t),
            Coup::RestoCoup(r) => r.delta(t),
        }
    }
}

//Implementación manual de Component para CoupAtom
unsafe impl Component for CoupAtom {
    type Input = ModCoupLIInput;

    type Output = ModCoupLIOutput;

    type InputRef<'a>
        = &'a mut Self::Input
    where
        Self: 'a;

    type OutputRef<'a>
        = &'a Self::Output
    where
        Self: 'a;

    fn get_t_last(&self) -> f64 {
        self.coup_atomic.get_t_last()
    }

    fn set_t_last(&mut self, t_last: f64) {
        self.coup_atomic.set_t_last(t_last);
    }

    fn get_t_next(&self) -> f64 {
        self.coup_atomic.get_t_next()
    }

    fn set_t_next(&mut self, t_next: f64) {
        self.coup_atomic.set_t_next(t_next)
    }

    fn get_input(&self) -> &Self::Input {
        self.coup_atomic.get_input()
    }

    fn get_input_mut(&mut self) -> &mut Self::Input {
        self.coup_atomic.get_input_mut()
    }

    fn get_output(&self) -> &Self::Output {
        self.coup_atomic.get_output()
    }

    fn get_output_mut(&mut self) -> &mut Self::Output {
        self.coup_atomic.get_output_mut()
    }

    fn get_ports(&mut self) -> (Self::InputRef<'_>, Self::OutputRef<'_>) {
        self.coup_atomic.get_ports()
    }

    fn get_out_ports(&self) -> Self::OutputRef<'_> {
        self.coup_atomic.get_out_ports()
    }
}

//Implementación manual de Component para Coup (porque AbstractSimulator requiere component)
unsafe impl<const W: usize, const P: usize> Component for Coup<W, P> {
    type Input = ModCoupLIInput; //si lo ponemos así el compilador se raya por los genéricos. Como sabemos qué tipo es, poner directamente
    type Output = ModCoupLIOutput;

    type InputRef<'a>
        = &'a mut Self::Input
    where
        Self: 'a;
    type OutputRef<'a>
        = &'a Self::Output
    where
        Self: 'a;

    fn get_t_last(&self) -> f64 {
        match self {
            Coup::CoupD(d) => d.get_t_last(),
            Coup::RestoCoup(r) => r.get_t_last(),
        }
    }

    fn set_t_last(&mut self, _t_last: f64) {
        match self {
            Coup::CoupD(d) => d.set_t_last(_t_last),
            Coup::RestoCoup(r) => r.set_t_last(_t_last),
        }
    }

    fn get_t_next(&self) -> f64 {
        match self {
            Coup::CoupD(d) => d.get_t_next(),
            Coup::RestoCoup(r) => r.get_t_next(),
        }
    }

    fn set_t_next(&mut self, _t_next: f64) {
        match self {
            Coup::CoupD(d) => d.set_t_next(_t_next),
            Coup::RestoCoup(r) => r.set_t_next(_t_next),
        }
    }

    fn get_input(&self) -> &Self::Input {
        match self {
            //Coup::CoupD(d) => interfaz_puerto_input(d.get_input()),
            Coup::CoupD(d) => d.get_input(),
            Coup::RestoCoup(r) => r.get_input(),
        }
    }

    fn get_input_mut(&mut self) -> &mut Self::Input {
        match self {
            //Coup::CoupD(d) => interfaz_puerto_input(d.get_input_mut()),
            Coup::CoupD(d) => d.get_input_mut(),
            Coup::RestoCoup(r) => r.get_input_mut(),
        }
    }

    fn get_output(&self) -> &Self::Output {
        match self {
            Coup::CoupD(d) => d.get_output(),
            //Coup::CoupD(d) => interfaz_puerto_output(d.get_output()),
            Coup::RestoCoup(r) => r.get_output(),
        }
    }

    fn get_output_mut(&mut self) -> &mut Self::Output {
        match self {
            Coup::CoupD(d) => d.get_output_mut(),
            //Coup::CoupD(d) => interfaz_puerto_output(d.get_output_mut()),
            Coup::RestoCoup(r) => r.get_output_mut(),
        }
    }

    fn get_ports(&mut self) -> (Self::InputRef<'_>, Self::OutputRef<'_>) {
        match self {
            Coup::CoupD(d) => d.get_ports(),
            //Coup::CoupD(d) => interfaz_puertos(d.get_ports()),
            Coup::RestoCoup(r) => r.get_ports(),
        }
    }

    fn get_out_ports(&self) -> Self::OutputRef<'_> {
        match self {
            Coup::CoupD(d) => d.get_out_ports(),
            //Coup::CoupD(d) => interfaz_puerto_output(d.get_out_ports()),
            Coup::RestoCoup(r) => r.get_out_ports(),
        }
    }
}

//Implementación manual de Coupled para ModCoupLI porque la macro no lo implementa
impl<const W: usize, const P: usize> xdevs::Coupled for ModCoupLI<W, P> {
    /// External Input Coupling. Propagates input events from the coupled model to its inner components.
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput<'_>) {
        if let Some(&value) = from.input_port.get_values().last() {
            for atom_input in to.component_li.iter_mut() {
                atom_input.input_port.add_value(value).unwrap();
            }
            // for coup_input in to.comp_coupled.iter_mut() {
            //     coup_input.input_port.add_value(value).unwrap();
            // }
        }
    }

    // External Output Coupling. Propagates output events from inner components to the coupled model's output.
    fn eoc(from: &Self::ComponentsOutput<'_>, to: &mut Self::Output) {
        // from.comp_coupled
        //     .output_port
        //     .couple(&mut to.output_port)
        //     .unwrap();
        for value in from.component_li.output_port.get_values() {
            to.output_port.add_value(*value).unwrap();
        }
    }
}

fn main() {
    let period = 1.0;
    let mut atom = CoupAtom::new(period);
    //let mut coup = Coup::new(Coup < 3, 3 > (), period);
    let mut coup1 = Coup::CoupD(atom);
    let coup1 = Box::new(coup1);
    let mut coup_acoplados = Coup::RestoCoup(coup1);

    let mut modelo_li: ModCoupLI<_, _> = ModCoupLI::build(Box::new(coup));

    let mut simulator = xdevs::simulator::Simulator::new(modelo_li);
    let config = xdevs::simulator::Config::new(0.0, 10.0, 1.0, None);
    simulator.simulate_rt(&config, xdevs::simulator::std::sleep(&config), |_| {});
}
