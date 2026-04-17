use core::f64;

use xdevs::traits::{AbstractSimulator, Component};

static mut N_EIC: usize = 0;
static mut N_EOC: usize = 0;
// static mut N_IC: usize = 0;

//Atomic model

#[xdevs::atomic]
pub struct Atom {
    #[input]
    input_port: xdevs::port::Port<usize, 1>,
    #[output]
    output_port: xdevs::port::Port<usize, 1>,
    #[state]
    sigma: f64,
    period: f64,
    n_internals: usize,
    n_externals: usize, //debería ser igual que n_internals
    n_events: usize,    //número de eventos que llegan al puerto acumulador
}

impl xdevs::Atomic for Atom {
    fn delta_int(state: &mut Self::State) {
        state.sigma = f64::INFINITY;
        state.n_internals += 1;
        println!("Número de deltas internas: {}", state.n_internals);
    }

    fn lambda(state: &Self::State, output: &mut Self::Output) {
        output.output_port.add_value(state.n_events).unwrap();
    }

    fn ta(state: &Self::State) -> f64 {
        state.sigma
    }

    fn delta_ext(state: &mut Self::State, elapsed: f64, input: &Self::Input) {
        // state.sigma -= elapsed;
        state.sigma = 0.0;
        state.n_externals += 1;
        state.n_events += input.input_port.get_values().len();
        println!("Número de deltas externas: {}", state.n_externals);
    }
}

impl Atom {
    pub fn new(period: f64) -> Self {
        Self::build(f64::INFINITY, period, 0, 0, 0)
    }

    pub fn get_count(&self) -> usize {
        self.state.n_internals
    }
}
//Fin atomic model

/*ENUM: hay 2 opciones
- Opción 1: acoplado con un único atomic
- Opción 2: acoplado con un atomic y un acoplado, y el acoplado a su vez con un atomic y un acoplado, etc. (estructura recursiva)
*/

//Inicio modelo acoplado CoupAtom que contiene un único atómico
// pub struct CoupAtom {
//     coup_atomic: Atom,
// }

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
    pub input: ModCoupLIInput,
    pub output: ModCoupLIOutput,
    pub t_last: f64,
    pub t_next: f64,
    pub components: CoupAtomComponents,
}
impl CoupAtom {
    #[inline]
    pub fn build(coup_atomic: Atom) -> Self {
        Self {
            input: ModCoupLIInput::new(),
            output: ModCoupLIOutput::new(),
            t_last: 0.0,
            t_next: f64::INFINITY,
            components: CoupAtomComponents::new(coup_atomic),
        }
    }
}
unsafe impl xdevs::traits::Component for CoupAtom {
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
    pub fn new(period: f64) -> Self {
        Self::build(Atom::new(period))
    }

    pub fn get_count(&self) -> usize {
        self.components.coup_atomic.get_count()
    }
}

impl xdevs::Coupled for CoupAtom {
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput<'_>) {
        from.input_port.couple(&mut to.coup_atomic.input_port);
        let port = &from.input_port;

        if !port.is_empty() {
            unsafe {
                N_EIC += 1;
                println!("Número de eventos EIC: {}", N_EIC);
            }
        }
    }
    fn eoc(from: &Self::ComponentsOutput<'_>, to: &mut Self::Output) {
        from.coup_atomic.output_port.couple(&mut to.output_port);
        unsafe {
            N_EOC += 1;
            println!("Número de eventos EOC: {}", N_EOC);
        }
    }
    fn ic(from: &Self::ComponentsOutput<'_>, to: &mut Self::ComponentsInput<'_>) {
        // unsafe {
        //     // N_IC += 1;
        //     println!("Número de eventos IC: {}", N_IC);
        // }
    }
}

//Fin modelo acoplado CoupAtom que contiene un único atómico

//Inicio modelo acoplado Coup
// pub enum Coup<const W: usize> {
//     CoupD(Atom), //en vez de Atom, Coup (como no es recursivo no hace falta pasarlo por referencia)
//     RestoCoup(Box<Self>), //RestoCoup(Box<Coup<W>>),
// }

pub enum Coup<const W: usize> {
    CoupD(CoupAtom),
    RestoCoup(ModCoupLI<W>),
}

impl<const W: usize> Coup<W> {
    pub fn get_count(&self) -> usize {
        match self {
            Coup::CoupD(coup_atom) => coup_atom.get_count(),
            Coup::RestoCoup(mod_coup_li) => mod_coup_li.get_count(),
        }
    }
}

//REVISAR
// impl<const W: usize> Coup<W> {
//     fn new(&self, period: f64) -> Self {
//         match self {
//             Coup::CoupD(coup_atom) => Coup::CoupD(CoupAtom::new(period)),
//             Coup::RestoCoup(coup) => Coup::RestoCoup(Box::new(Coup::new(coup, period))),
//         }
//     }
// }
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
// pub struct ModCoupLI<const W: usize> {
//     #[input]
//     input_port: xdevs::port::Port<usize, 1>,
//     #[output]
//     output_port: xdevs::port::Port<usize, 1>,
//     #[components]
//     //constante con la width y otra que sea la width-1 para los genéricos
//     comp_atomic: [Atom; W],
//     comp_coupled: Box<Coup<W>>,
// }

// Recursive expansion of coupled2 macro
// ======================================

#[derive(Debug, Default)]
pub struct ModCoupLIInput {
    pub input_port: xdevs::port::Port<usize, 1>,
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
pub struct ModCoupLIComponents<const W: usize> {
    comp_atomic: [Atom; W],
    comp_coupled: Box<Coup<W>>,
}
impl<const W: usize> ModCoupLIComponents<W> {
    #[inline]
    pub fn new(comp_atomic: [Atom; W], comp_coupled: Box<Coup<W>>) -> Self {
        Self {
            comp_atomic,
            comp_coupled,
        }
    }
}
#[doc = r" Wrapper struct holding mutable references to all inner components' inputs."]
pub struct ModCoupLIComponentsInput<'__xdevs_inner, const W: usize> {
    pub comp_atomic: <[Atom; W] as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
    pub comp_coupled: <Box<Coup<W>> as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
}
#[doc = r" Wrapper struct holding references to all inner components' outputs."]
pub struct ModCoupLIComponentsOutput<'__xdevs_inner, const W: usize> {
    pub comp_atomic: <[Atom; W] as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
    pub comp_coupled: <Box<Coup<W>> as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
}
pub struct ModCoupLI<const W: usize> {
    pub input: ModCoupLIInput,
    pub output: ModCoupLIOutput,
    pub t_last: f64,
    pub t_next: f64,
    pub components: ModCoupLIComponents<W>,
}
impl<const W: usize> ModCoupLI<W> {
    #[inline]
    pub fn build(comp_atomic: [Atom; W], comp_coupled: Box<Coup<W>>) -> Self {
        Self {
            input: ModCoupLIInput::new(),
            output: ModCoupLIOutput::new(),
            t_last: 0.0,
            t_next: f64::INFINITY,
            components: ModCoupLIComponents::new(comp_atomic, comp_coupled),
        }
    }
}
unsafe impl<const W: usize> xdevs::traits::Component for ModCoupLI<W> {
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
unsafe impl<const W: usize> xdevs::traits::PartialCoupled for ModCoupLI<W> {
    type ComponentsInput<'__xdevs_inner>
        = ModCoupLIComponentsInput<'__xdevs_inner, W>
    where
        Self: '__xdevs_inner;
    type ComponentsOutput<'__xdevs_inner>
        = ModCoupLIComponentsOutput<'__xdevs_inner, W>
    where
        Self: '__xdevs_inner;
}
unsafe impl<const W: usize> xdevs::traits::AbstractSimulator for ModCoupLI<W> {
    #[inline]
    fn start(&mut self, t_start: f64) -> f64 {
        xdevs::traits::Component::set_t_last(self, t_start);
        let mut t_next = f64::INFINITY;
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::start(&mut self.components.comp_atomic, t_start),
        );
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::start(&mut self.components.comp_coupled, t_start),
        );
        xdevs::traits::Component::set_t_next(self, t_next);
        t_next
    }
    #[inline]
    fn stop(&mut self, t_stop: f64) {
        xdevs::traits::AbstractSimulator::stop(&mut self.components.comp_atomic, t_stop);
        xdevs::traits::AbstractSimulator::stop(&mut self.components.comp_coupled, t_stop);
        xdevs::traits::Component::set_t_last(self, t_stop);
        xdevs::traits::Component::set_t_next(self, f64::INFINITY);
    }
    #[inline]
    fn lambda(&mut self, t: f64) {
        if t >= xdevs::traits::Component::get_t_next(self) {
            xdevs::traits::AbstractSimulator::lambda(&mut self.components.comp_atomic, t);
            xdevs::traits::AbstractSimulator::lambda(&mut self.components.comp_coupled, t);
            let comp_atomic_output =
                xdevs::traits::Component::get_out_ports(&self.components.comp_atomic);
            let comp_coupled_output =
                xdevs::traits::Component::get_out_ports(&self.components.comp_coupled);
            let component_outputs: ModCoupLIComponentsOutput<'_, W> = ModCoupLIComponentsOutput {
                comp_atomic: comp_atomic_output,
                comp_coupled: comp_coupled_output,
            };
            <Self as xdevs::Coupled>::eoc(&component_outputs, &mut self.output);
        }
    }
    #[inline]
    fn delta(&mut self, t: f64) -> f64 {
        {
            let (comp_atomic_input, comp_atomic_output) =
                xdevs::traits::Component::get_ports(&mut self.components.comp_atomic);
            let (comp_coupled_input, comp_coupled_output) =
                xdevs::traits::Component::get_ports(&mut self.components.comp_coupled);
            let component_outputs: ModCoupLIComponentsOutput<'_, W> = ModCoupLIComponentsOutput {
                comp_atomic: comp_atomic_output,
                comp_coupled: comp_coupled_output,
            };
            let mut component_inputs: ModCoupLIComponentsInput<'_, W> = ModCoupLIComponentsInput {
                comp_atomic: comp_atomic_input,
                comp_coupled: comp_coupled_input,
            };
            <Self as xdevs::Coupled>::eic(&self.input, &mut component_inputs);
            <Self as xdevs::Coupled>::ic(&component_outputs, &mut component_inputs);
        }
        let mut t_next = f64::INFINITY;
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::delta(&mut self.components.comp_atomic, t),
        );
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::delta(&mut self.components.comp_coupled, t),
        );
        xdevs::traits::Component::clear_output(self);
        xdevs::traits::Component::clear_input(self);
        xdevs::traits::Component::set_t_last(self, t);
        xdevs::traits::Component::set_t_next(self, t_next);
        t_next
    }
}

impl<const W: usize> ModCoupLI<W> {
    pub fn get_count(&self) -> usize {
        let mut sum = self.components.comp_coupled.get_count(); //implementar también para el enum
        for atomic in self.components.comp_atomic.iter() {
            //inmuable como self
            sum += atomic.get_count();
        }
        sum
    }
}
//Fin modelo LI CoupModLI

//Implementación manual de AbstracSimulator para CoupAtom
// unsafe impl AbstractSimulator for CoupAtom {
//     fn start(&mut self, t_start: f64) -> f64 {
//         self.coup_atomic.start(t_start)
//     }

//     fn stop(&mut self, t_stop: f64) {
//         self.coup_atomic.stop(t_stop)
//     }

//     fn lambda(&mut self, t: f64) {
//         self.coup_atomic.lambda(t)
//     }

//     fn delta(&mut self, t: f64) -> f64 {
//         self.coup_atomic.delta(t)
//     }
// }
//Implementación manual de AbstracSimulator para Coup (la macro no lo implementa)
unsafe impl<const W: usize> AbstractSimulator for Coup<W> {
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
// unsafe impl Component for CoupAtom {
//     type Input = ModCoupLIInput;

//     type Output = ModCoupLIOutput;

//     type InputRef<'a>
//         = &'a mut Self::Input
//     where
//         Self: 'a;

//     type OutputRef<'a>
//         = &'a Self::Output
//     where
//         Self: 'a;

//     fn get_t_last(&self) -> f64 {
//         self.coup_atomic.get_t_last()
//     }

//     fn set_t_last(&mut self, t_last: f64) {
//         self.coup_atomic.set_t_last(t_last);
//     }

//     fn get_t_next(&self) -> f64 {
//         self.coup_atomic.get_t_next()
//     }

//     fn set_t_next(&mut self, t_next: f64) {
//         self.coup_atomic.set_t_next(t_next)
//     }

//     fn get_input(&self) -> &Self::Input {
//         self.coup_atomic.get_input()
//     }

//     fn get_input_mut(&mut self) -> &mut Self::Input {
//         self.coup_atomic.get_input_mut()
//     }

//     fn get_output(&self) -> &Self::Output {
//         self.coup_atomic.get_output()
//     }

//     fn get_output_mut(&mut self) -> &mut Self::Output {
//         self.coup_atomic.get_output_mut()
//     }

//     fn get_ports(&mut self) -> (Self::InputRef<'_>, Self::OutputRef<'_>) {
//         self.coup_atomic.get_ports()
//     }

//     fn get_out_ports(&self) -> Self::OutputRef<'_> {
//         self.coup_atomic.get_out_ports()
//     }
// }

//Implementación manual de Component para Coup (porque AbstractSimulator requiere component)
unsafe impl<const W: usize> Component for Coup<W> {
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
impl<const W: usize> xdevs::Coupled for ModCoupLI<W> {
    /// External Input Coupling. Propagates input events from the coupled model to its inner components.
    // Iteración para la conexión con los atómicos
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput<'_>) {
        for atom_ports in to.comp_atomic.iter_mut() {
            from.input_port.couple(&mut atom_ports.input_port).unwrap();
        }

        from.input_port //Conexión con el coupled
            .couple(&mut to.comp_coupled.input_port)
            .unwrap();

        let port = &from.input_port;

        if !port.is_empty() {
            unsafe {
                N_EIC += 1;
                println!("Número de eventos EIC: {}", N_EIC);
            }
        }
    }

    // External Output Coupling. Propagates output events from inner components to the coupled model's output.
    fn eoc(from: &Self::ComponentsOutput<'_>, to: &mut Self::Output) {
        // from.comp_coupled
        //     .output_port
        //     .couple(&mut to.output_port)
        //     .unwrap();
        from.comp_coupled
            .output_port
            .couple(&mut to.output_port)
            .unwrap();
        let port = &from.comp_coupled.output_port;
        if port.is_empty() {
            println!("Puerto de salida del coupled está vacío");
        } else {
            println!(
                "Puerto de salida del coupled tiene valores: {:?}",
                port.get_values()
            );
            unsafe {
                N_EOC += 1;
                println!("Número de eventos EOC: {}", N_EOC);
            }
        }
    }

    fn ic(from: &Self::ComponentsOutput<'_>, to: &mut Self::ComponentsInput<'_>) {
        // unsafe {
        //     N_IC += 1;
        //     println!("Número de eventos IC: {}", N_IC);
        // }
    }
}

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
        println!("Generador ha generado el valor: {}", state.count);
    }

    fn ta(state: &Self::State) -> f64 {
        state.sigma
    }

    fn delta_ext(state: &mut Self::State, elapsed: f64, input: &Self::Input) {}
}

impl Generator {
    pub fn new(val_count: usize) -> Self {
        Self::build(0.0, val_count)
    }
}
//Fin modelo atómico sencillo que mete datos en el puerto de entrada del modelo LI

//Modelo acoplado ModeloFinal que recibe los datos de Generator y los introduce en el puerto de entrada del modelo LI
// #[xdevs::coupled2]
// pub struct ModeloFinal<const W: usize> {
//     #[components]
//     generator: Generator,
//     modelo_li: Coup<W>,
// }

// Recursive expansion of coupled2 macro
// ======================================

#[derive(Debug, Default)]
pub struct ModeloFinalInput {}

impl ModeloFinalInput {
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }
}
unsafe impl xdevs::traits::Bag for ModeloFinalInput {
    #[inline]
    fn is_empty(&self) -> bool {
        true
    }
    #[inline]
    fn clear(&mut self) {}
}
#[derive(Debug, Default)]
pub struct ModeloFinalOutput {}

impl ModeloFinalOutput {
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }
}
unsafe impl xdevs::traits::Bag for ModeloFinalOutput {
    #[inline]
    fn is_empty(&self) -> bool {
        true
    }
    #[inline]
    fn clear(&mut self) {}
}
pub struct ModeloFinalComponents<const W: usize> {
    generator: Generator,
    modelo_li: Coup<W>,
}
impl<const W: usize> ModeloFinalComponents<W> {
    #[inline]
    pub fn new(generator: Generator, modelo_li: Coup<W>) -> Self {
        Self {
            generator,
            modelo_li,
        }
    }
}
#[doc = r" Wrapper struct holding mutable references to all inner components' inputs."]
pub struct ModeloFinalComponentsInput<'__xdevs_inner, const W: usize> {
    pub generator: <Generator as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
    pub modelo_li: <Coup<W> as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
}
#[doc = r" Wrapper struct holding references to all inner components' outputs."]
pub struct ModeloFinalComponentsOutput<'__xdevs_inner, const W: usize> {
    pub generator: <Generator as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
    pub modelo_li: <Coup<W> as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
}
pub struct ModeloFinal<const W: usize> {
    pub input: ModeloFinalInput,
    pub output: ModeloFinalOutput,
    pub t_last: f64,
    pub t_next: f64,
    pub components: ModeloFinalComponents<W>,
}
impl<const W: usize> ModeloFinal<W> {
    #[inline]
    pub fn build(generator: Generator, modelo_li: Coup<W>) -> Self {
        Self {
            input: ModeloFinalInput::new(),
            output: ModeloFinalOutput::new(),
            t_last: 0.0,
            t_next: f64::INFINITY,
            components: ModeloFinalComponents::new(generator, modelo_li),
        }
    }
}
unsafe impl<const W: usize> xdevs::traits::Component for ModeloFinal<W> {
    type Input = ModeloFinalInput;
    type Output = ModeloFinalOutput;
    type InputRef<'__xdevs_ports>
        = &'__xdevs_ports mut ModeloFinalInput
    where
        Self: '__xdevs_ports;
    type OutputRef<'__xdevs_ports>
        = &'__xdevs_ports ModeloFinalOutput
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
unsafe impl<const W: usize> xdevs::traits::PartialCoupled for ModeloFinal<W> {
    type ComponentsInput<'__xdevs_inner>
        = ModeloFinalComponentsInput<'__xdevs_inner, W>
    where
        Self: '__xdevs_inner;
    type ComponentsOutput<'__xdevs_inner>
        = ModeloFinalComponentsOutput<'__xdevs_inner, W>
    where
        Self: '__xdevs_inner;
}
unsafe impl<const W: usize> xdevs::traits::AbstractSimulator for ModeloFinal<W> {
    #[inline]
    fn start(&mut self, t_start: f64) -> f64 {
        xdevs::traits::Component::set_t_last(self, t_start);
        let mut t_next = f64::INFINITY;
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::start(&mut self.components.generator, t_start),
        );
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::start(&mut self.components.modelo_li, t_start),
        );
        xdevs::traits::Component::set_t_next(self, t_next);
        t_next
    }
    #[inline]
    fn stop(&mut self, t_stop: f64) {
        xdevs::traits::AbstractSimulator::stop(&mut self.components.generator, t_stop);
        xdevs::traits::AbstractSimulator::stop(&mut self.components.modelo_li, t_stop);
        xdevs::traits::Component::set_t_last(self, t_stop);
        xdevs::traits::Component::set_t_next(self, f64::INFINITY);
    }
    #[inline]
    fn lambda(&mut self, t: f64) {
        if t >= xdevs::traits::Component::get_t_next(self) {
            xdevs::traits::AbstractSimulator::lambda(&mut self.components.generator, t);
            xdevs::traits::AbstractSimulator::lambda(&mut self.components.modelo_li, t);
            let generator_output =
                xdevs::traits::Component::get_out_ports(&self.components.generator);
            let modelo_li_output =
                xdevs::traits::Component::get_out_ports(&self.components.modelo_li);
            let component_outputs: ModeloFinalComponentsOutput<'_, W> =
                ModeloFinalComponentsOutput {
                    generator: generator_output,
                    modelo_li: modelo_li_output,
                };
            <Self as xdevs::Coupled>::eoc(&component_outputs, &mut self.output);
        }
    }
    #[inline]
    fn delta(&mut self, t: f64) -> f64 {
        {
            let (generator_input, generator_output) =
                xdevs::traits::Component::get_ports(&mut self.components.generator);
            let (modelo_li_input, modelo_li_output) =
                xdevs::traits::Component::get_ports(&mut self.components.modelo_li);
            let component_outputs: ModeloFinalComponentsOutput<'_, W> =
                ModeloFinalComponentsOutput {
                    generator: generator_output,
                    modelo_li: modelo_li_output,
                };
            let mut component_inputs: ModeloFinalComponentsInput<'_, W> =
                ModeloFinalComponentsInput {
                    generator: generator_input,
                    modelo_li: modelo_li_input,
                };
            <Self as xdevs::Coupled>::eic(&self.input, &mut component_inputs);
            <Self as xdevs::Coupled>::ic(&component_outputs, &mut component_inputs);
        }
        let mut t_next = f64::INFINITY;
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::delta(&mut self.components.generator, t),
        );
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::delta(&mut self.components.modelo_li, t),
        );
        xdevs::traits::Component::clear_output(self);
        xdevs::traits::Component::clear_input(self);
        xdevs::traits::Component::set_t_last(self, t);
        xdevs::traits::Component::set_t_next(self, t_next);
        t_next
    }
}

//Implementación manual de Coupled para ModeloFinal
impl<const W: usize> xdevs::Coupled for ModeloFinal<W> {
    fn ic(from: &Self::ComponentsOutput<'_>, to: &mut Self::ComponentsInput<'_>) {
        from.generator
            .out_job
            .couple(&mut to.modelo_li.input_port)
            .unwrap();
    }
}

// #[xdevs::coupled(
//     couplings = {generator.out_job -> modelo_li.input_port}
// )
// ]

// struct ModeloFinal<const W: usize> {
//     #[components]
//     generator: Generator,
//     modelo_li: ModCoupLI<W>,
// }

fn main() {
    const WIDTH: usize = 1; //a lo mejor me toca sacar WIDTH y DEPTH del main y hacerlas globales para que los tests puedan usarlas
    const DEPTH: usize = 2;
    const W: usize = WIDTH - 1;
    //Creación de modelo LI con W = 2 (según el modelo teórico sería W = 3, con W-1 atómicos cada acoplado)
    let period = 10.0;
    let atom = CoupAtom::new(period); //Modelo atómico que va dentro del acoplado (es el acoplado más interno del modelo LI, es CoupD)
    let coup_atom_d: Coup<W> = Coup::CoupD(atom);
    let modelo_li: ModCoupLI<W> = ModCoupLI::build(
        core::array::from_fn(|_| Atom::new(period)),
        Box::new(coup_atom_d),
    );
    // let modelo_li_2: ModCoupLI<W> = ModCoupLI::build(
    //     core::array::from_fn(|_| Atom::new(period)),
    //     Box::new(Coup::RestoCoup(modelo_li)),
    // );
    // let modelo_li_3: ModCoupLI<W> = ModCoupLI::build(
    //     core::array::from_fn(|_| Atom::new(period)),
    //     Box::new(Coup::RestoCoup(modelo_li_2)),
    // );
    // let modelo_li_4: ModCoupLI<W> = ModCoupLI::build(
    //     core::array::from_fn(|_| Atom::new(period)),
    //     Box::new(Coup::RestoCoup(modelo_li_3)),
    // );

    //Creación del modelo atómico generador (mete datos en el modelo LI)
    let generator = Generator::new(5);

    //Creación del modelo final (modelo LI + atómico generador que mete datos en el puerto del LI)
    let modelo_final = ModeloFinal::build(generator, Coup::RestoCoup(modelo_li));

    let mut simulator = xdevs::simulator::Simulator::new(modelo_final);
    let config = xdevs::simulator::Config::new(0.0, 10.0, 1.0, None);
    simulator.simulate_rt(&config, xdevs::simulator::std::sleep(&config), |_| {});
}
