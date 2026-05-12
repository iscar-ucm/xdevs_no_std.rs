use crate::common::*;
use xdevs::traits::{AbstractSimulator, Component};

pub enum Coup<const W: usize> {
    CoupD(CoupAtom),
    RestoCoup(ModCoupLI<W>),
}

impl<const W: usize> Coup<W> {
    pub fn get_n_internals(&self) -> usize {
        match self {
            Coup::CoupD(coup_atom) => coup_atom.get_n_internals(),
            Coup::RestoCoup(mod_coup_li) => mod_coup_li.get_n_internals(),
        }
    }

    pub fn get_n_externals(&self) -> usize {
        match self {
            Coup::CoupD(coup_atom) => coup_atom.get_n_externals(),
            Coup::RestoCoup(mod_coup_li) => mod_coup_li.get_n_externals(),
        }
    }

    pub fn get_n_events(&self) -> usize {
        match self {
            Coup::CoupD(coup_atom) => coup_atom.get_n_events(),
            Coup::RestoCoup(mod_coup_li) => mod_coup_li.get_n_events(),
        }
    }

    pub fn get_n_atomics(&self) -> usize {
        match self {
            Coup::CoupD(coup_atom) => coup_atom.get_n_atomics(),
            Coup::RestoCoup(mod_coup_li) => mod_coup_li.get_n_atomics(),
        }
    }
}

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

//Implementación manual de Component para Coup (porque AbstractSimulator requiere component)
unsafe impl<const W: usize> Component for Coup<W> {
    type Input = CoupInputPort; //si lo ponemos así el compilador se raya por los genéricos. Como sabemos qué tipo es, poner directamente
    type Output = CoupOutputPort;

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
//Fin enum

//Inicio del acoplado con un array de atómicos y otro acoplado igual
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
    pub input: CoupInputPort,
    pub output: CoupOutputPort,
    pub t_last: f64,
    pub t_next: f64,
    pub components: ModCoupLIComponents<W>,
}
impl<const W: usize> ModCoupLI<W> {
    #[inline]
    pub fn build(comp_atomic: [Atom; W], comp_coupled: Box<Coup<W>>) -> Self {
        Self {
            input: CoupInputPort::new(),
            output: CoupOutputPort::new(),
            t_last: 0.0,
            t_next: f64::INFINITY,
            components: ModCoupLIComponents::new(comp_atomic, comp_coupled),
        }
    }
}
unsafe impl<const W: usize> xdevs::traits::Component for ModCoupLI<W> {
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
    pub fn new(coup: Box<Coup<W>>) -> Self {
        Self::build(core::array::from_fn(|_| Atom::new()), coup)
    }

    pub fn get_n_internals(&self) -> usize {
        let mut sum_int = self.components.comp_coupled.get_n_internals(); //implementar también para el enum
        for atomic in self.components.comp_atomic.iter() {
            //inmutable como self
            sum_int += atomic.get_n_internals();
        }
        sum_int
    }

    pub fn get_n_externals(&self) -> usize {
        let mut sum_ext = self.components.comp_coupled.get_n_externals();
        for atomic in self.components.comp_atomic.iter() {
            sum_ext += atomic.get_n_externals();
        }
        sum_ext
    }

    pub fn get_n_events(&self) -> usize {
        let mut sum_ev = self.components.comp_coupled.get_n_events();
        for atomic in self.components.comp_atomic.iter() {
            sum_ev += atomic.get_n_events();
        }
        sum_ev
    }

    pub fn get_n_atomics(&self) -> usize {
        let mut sum_atomic = self.components.comp_coupled.get_n_atomics();
        for _atomic in self.components.comp_atomic.iter() {
            sum_atomic += 1;
        }
        sum_atomic
    }
}

//Implementación manual de Coupled para ModCoupLI porque la macro no lo implementa
impl<const W: usize> xdevs::Coupled for ModCoupLI<W> {
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput<'_>) {
        for atom_ports in to.comp_atomic.iter_mut() {
            from.input_port.couple(&mut atom_ports.input_port).unwrap();
        }

        from.input_port //Conexión con el coupled
            .couple(&mut to.comp_coupled.input_port)
            .unwrap();
    }

    fn eoc(from: &Self::ComponentsOutput<'_>, to: &mut Self::Output) {
        from.comp_coupled
            .output_port
            .couple(&mut to.output_port)
            .unwrap();
    }
}
//Fin del acoplado con un array de atómicos y otro acoplado igual

//Inicio modelo acoplado ModeloFinal que recibe los datos de Generator y los introduce en el puerto de entrada del modelo LI
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

    pub fn get_n_internals(&self) -> usize {
        self.components.modelo_li.get_n_internals()
    }

    pub fn get_n_externals(&self) -> usize {
        self.components.modelo_li.get_n_externals()
    }

    pub fn get_n_events(&self) -> usize {
        self.components.modelo_li.get_n_events()
    }

    pub fn get_n_atomics(&self) -> usize {
        self.components.modelo_li.get_n_atomics()
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
//Fin modelo acoplado ModeloFinal que recibe los datos de Generator y los introduce en el puerto de entrada del modelo LI
