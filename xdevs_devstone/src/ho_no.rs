use crate::common::*;
use xdevs::traits::{AbstractSimulator, Component};

//Inicio enum con las opciones que puede haber en el modelo HO
pub enum HO<const W: usize> {
    CoupD(CoupAtom),
    RestoCoup(CoupHO<W>),
}

//Implementacion manual de AbstractSimulator para HO
unsafe impl<const W: usize> AbstractSimulator for HO<W> {
    fn start(&mut self, t_start: f64) -> f64 {
        match self {
            HO::CoupD(coup_d) => coup_d.start(t_start),
            HO::RestoCoup(coup_r) => coup_r.start(t_start),
        }
    }

    fn stop(&mut self, t_stop: f64) {
        match self {
            HO::CoupD(coup_d) => coup_d.stop(t_stop),
            HO::RestoCoup(coup_r) => coup_r.stop(t_stop),
        }
    }

    fn lambda(&mut self, t: f64) {
        match self {
            HO::CoupD(coup_d) => coup_d.lambda(t),
            HO::RestoCoup(coup_r) => coup_r.lambda(t),
        }
    }

    fn delta(&mut self, t: f64) -> f64 {
        match self {
            HO::CoupD(coup_d) => coup_d.delta(t),
            HO::RestoCoup(coup_r) => coup_r.delta(t),
        }
    }
}

//Implementación manual de Component para HO
unsafe impl<const W: usize> Component for HO<W> {
    type Input = CoupInputPort;

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
            HO::CoupD(coup_d) => coup_d.get_t_last(),
            HO::RestoCoup(coup_r) => coup_r.get_t_last(),
        }
    }

    fn set_t_last(&mut self, t_last: f64) {
        match self {
            HO::CoupD(coup_d) => coup_d.set_t_last(t_last),
            HO::RestoCoup(coup_r) => coup_r.set_t_last(t_last),
        }
    }

    fn get_t_next(&self) -> f64 {
        match self {
            HO::CoupD(coup_d) => coup_d.get_t_next(),
            HO::RestoCoup(coup_r) => coup_r.get_t_next(),
        }
    }

    fn set_t_next(&mut self, t_next: f64) {
        match self {
            HO::CoupD(coup_d) => coup_d.set_t_next(t_next),
            HO::RestoCoup(coup_r) => coup_r.set_t_next(t_next),
        }
    }

    /// Returns a reference to the model's input event bag.
    fn get_input(&self) -> &Self::Input {
        match self {
            HO::CoupD(coup_d) => coup_d.get_input(),
            HO::RestoCoup(coup_r) => coup_r.get_input(),
        }
    }

    fn get_input_mut(&mut self) -> &mut Self::Input {
        match self {
            HO::CoupD(coup_d) => coup_d.get_input_mut(),
            HO::RestoCoup(coup_r) => coup_r.get_input_mut(),
        }
    }

    fn get_output(&self) -> &Self::Output {
        match self {
            HO::CoupD(coup_d) => coup_d.get_output(),
            // HO::RestoCoup(coup_r) => coup_r.get_output(),
            HO::RestoCoup(coup_r) => &coup_r.output.output_port_1,
        }
    }

    fn get_output_mut(&mut self) -> &mut Self::Output {
        match self {
            HO::CoupD(coup_d) => coup_d.get_output_mut(),
            // HO::RestoCoup(coup_r) => coup_r.get_output_mut(),
            HO::RestoCoup(coup_r) => &mut coup_r.output.output_port_1,
        }
    }

    fn get_ports(&mut self) -> (Self::InputRef<'_>, Self::OutputRef<'_>) {
        match self {
            HO::CoupD(coup_d) => coup_d.get_ports(),
            // HO::RestoCoup(coup_r) => coup_r.get_ports(),
            HO::RestoCoup(coup_r) => (&mut coup_r.input, &coup_r.output.output_port_1),
        }
    }

    fn get_out_ports(&self) -> Self::OutputRef<'_> {
        match self {
            HO::CoupD(coup_d) => coup_d.get_out_ports(),
            // HO::RestoCoup(coup_r) => coup_r.get_out_ports(),
            HO::RestoCoup(coup_r) => &coup_r.output.output_port_1,
        }
    }
}

impl<const W: usize> HO<W> {
    pub fn get_n_internals(&self) -> usize {
        match self {
            HO::CoupD(coup_atom) => coup_atom.get_n_internals(),
            HO::RestoCoup(coup_ho) => coup_ho.get_n_internals(),
        }
    }

    pub fn get_n_externals(&self) -> usize {
        match self {
            HO::CoupD(coup_atom) => coup_atom.get_n_externals(),
            HO::RestoCoup(coup_ho) => coup_ho.get_n_externals(),
        }
    }

    pub fn get_n_events(&self) -> usize {
        match self {
            HO::CoupD(coup_atom) => coup_atom.get_n_events(),
            HO::RestoCoup(coup_ho) => coup_ho.get_n_events(),
        }
    }

    pub fn get_n_atomics(&self) -> usize {
        match self {
            HO::CoupD(coup_atom) => coup_atom.get_n_atomics(),
            HO::RestoCoup(coup_ho) => coup_ho.get_n_atomics(),
        }
    }
}
//Fin enum con las opciones que puede haber en el modelo HO

//Inicio del acoplado con con un array de atómicos con puerto input tamaño 2 y otro acoplado igual
//1 puerto de entrada y 2 puertos de salida (uno de tamaño 1 y otro de tamaño W-1)
// #[xdevs::coupled2]
// pub struct CoupHO<const W: usize> {
//     #[input]
//     input_port: xdevs::port::Port<usize, 1>,
//     #[output]
//     output_port_1: xdevs::port::Port<usize, 1>,
//     output_port_2: xdevs::port::Port<usize, W>,
//     #[components]
//     comp_atomic: [AtomInputSize2; W],
//     comp_coupled: Box<HO<W>>,
// }

// Recursive expansion of coupled2 macro
// ======================================

// #[derive(Debug, Default)]
// pub struct CoupHOInput {
//     pub input_port: xdevs::port::Port<usize, 1>,
// }
// impl CoupHOInput {
//     #[inline]
//     pub const fn new() -> Self {
//         Self {
//             input_port: xdevs::port::Port::new(),
//         }
//     }
// }
// unsafe impl xdevs::traits::Bag for CoupHOInput {
//     #[inline]
//     fn is_empty(&self) -> bool {
//         true && self.input_port.is_empty()
//     }
//     #[inline]
//     fn clear(&mut self) {
//         self.input_port.clear();
//     }
// }
#[derive(Debug, Default)]
pub struct CoupHOOutput<const W: usize> {
    pub output_port_1: CoupOutputPort,
    pub output_port_2: xdevs::port::Port<usize, W>,
}
impl<const W: usize> CoupHOOutput<W> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            output_port_1: CoupOutputPort::new(),
            output_port_2: xdevs::port::Port::new(),
        }
    }
}
unsafe impl<const W: usize> xdevs::traits::Bag for CoupHOOutput<W> {
    #[inline]
    fn is_empty(&self) -> bool {
        true && self.output_port_1.is_empty() && self.output_port_2.is_empty()
    }
    #[inline]
    fn clear(&mut self) {
        self.output_port_1.clear();
        self.output_port_2.clear();
    }
}
pub struct CoupHOComponents<const W: usize> {
    comp_atomic: [AtomInputSize2; W],
    comp_coupled: Box<HO<W>>,
}
impl<const W: usize> CoupHOComponents<W> {
    #[inline]
    pub fn new(comp_atomic: [AtomInputSize2; W], comp_coupled: Box<HO<W>>) -> Self {
        Self {
            comp_atomic,
            comp_coupled,
        }
    }
}
#[doc = r" Wrapper struct holding mutable references to all inner components' inputs."]
pub struct CoupHOComponentsInput<'__xdevs_inner, const W: usize> {
    pub comp_atomic: <[AtomInputSize2; W] as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
    pub comp_coupled: <Box<HO<W>> as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
}
#[doc = r" Wrapper struct holding references to all inner components' outputs."]
pub struct CoupHOComponentsOutput<'__xdevs_inner, const W: usize> {
    pub comp_atomic: <[AtomInputSize2; W] as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
    pub comp_coupled: <Box<HO<W>> as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
}
pub struct CoupHO<const W: usize> {
    pub input: CoupInputPort,
    pub output: CoupHOOutput<W>,
    pub t_last: f64,
    pub t_next: f64,
    pub components: CoupHOComponents<W>,
}
impl<const W: usize> CoupHO<W> {
    #[inline]
    pub fn build(comp_atomic: [AtomInputSize2; W], comp_coupled: Box<HO<W>>) -> Self {
        Self {
            input: CoupInputPort::new(),
            output: CoupHOOutput::new(),
            t_last: 0.0,
            t_next: f64::INFINITY,
            components: CoupHOComponents::new(comp_atomic, comp_coupled),
        }
    }

    pub fn new(coup: Box<HO<W>>) -> Self {
        Self::build(core::array::from_fn(|_| AtomInputSize2::new()), coup)
    }

    pub fn get_n_internals(&self) -> usize {
        let mut sum_int = self.components.comp_coupled.get_n_internals();
        for atomic in self.components.comp_atomic.iter() {
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
unsafe impl<const W: usize> xdevs::traits::Component for CoupHO<W> {
    type Input = CoupInputPort;
    type Output = CoupHOOutput<W>;
    type InputRef<'__xdevs_ports>
        = &'__xdevs_ports mut CoupInputPort
    where
        Self: '__xdevs_ports;
    type OutputRef<'__xdevs_ports>
        = &'__xdevs_ports CoupHOOutput<W>
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
unsafe impl<const W: usize> xdevs::traits::PartialCoupled for CoupHO<W> {
    type ComponentsInput<'__xdevs_inner>
        = CoupHOComponentsInput<'__xdevs_inner, W>
    where
        Self: '__xdevs_inner;
    type ComponentsOutput<'__xdevs_inner>
        = CoupHOComponentsOutput<'__xdevs_inner, W>
    where
        Self: '__xdevs_inner;
}
unsafe impl<const W: usize> xdevs::traits::AbstractSimulator for CoupHO<W> {
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
            let component_outputs: CoupHOComponentsOutput<'_, W> = CoupHOComponentsOutput {
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
            let component_outputs: CoupHOComponentsOutput<'_, W> = CoupHOComponentsOutput {
                comp_atomic: comp_atomic_output,
                comp_coupled: comp_coupled_output,
            };
            let mut component_inputs: CoupHOComponentsInput<'_, W> = CoupHOComponentsInput {
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

//Implementación manual del trato Coupled
impl<const W: usize> xdevs::Coupled for CoupHO<W> {
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput<'_>) {
        from.input_port.couple(&mut to.comp_coupled.input_port);
        for atom_ports in to.comp_atomic.iter_mut() {
            from.input_port.couple(&mut atom_ports.input_port);
        }
    }

    fn eoc(from: &Self::ComponentsOutput<'_>, to: &mut Self::Output) {
        from.comp_coupled
            .output_port
            .couple(&mut to.output_port_1.output_port);
        for atom_output_ports in from.comp_atomic.iter() {
            atom_output_ports.output_port.couple(&mut to.output_port_2);
        }
    }

    fn ic(from: &Self::ComponentsOutput<'_>, to: &mut Self::ComponentsInput<'_>) {
        if W > 1 {
            for i in 0..(W - 1) {
                from.comp_atomic[i]
                    .output_port
                    .couple(&mut to.comp_atomic[i + 1].input_port);
            }
        }
    }
}

//Fin del acoplado con con un array de atómicos con puerto input tamaño 2 y otro acoplado igual

//Inicio acoplado ModeloFinal que recibe los datos de Generator y los introduce en el puerto de entrada del modelo HO
// #[xdevs::coupled2]
// pub struct ModeloFinal<const W: usize> {
//     #[input]
//     input_port: xdevs::port::Port<usize, 1>,
//     #[output]
//     output_port: xdevs::port::Port<usize, 1>,
//     #[components]
//     generator: Generator,
//     modelo_ho: HO<W>,
// }

// Recursive expansion of coupled2 macro
// ======================================

#[derive(Debug, Default)]
pub struct ModeloFinalInput {
    pub input_port: xdevs::port::Port<usize, 1>,
}
impl ModeloFinalInput {
    #[inline]
    pub const fn new() -> Self {
        Self {
            input_port: xdevs::port::Port::new(),
        }
    }
}
unsafe impl xdevs::traits::Bag for ModeloFinalInput {
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
pub struct ModeloFinalOutput {
    pub output_port: xdevs::port::Port<usize, 1>,
}
impl ModeloFinalOutput {
    #[inline]
    pub const fn new() -> Self {
        Self {
            output_port: xdevs::port::Port::new(),
        }
    }
}
unsafe impl xdevs::traits::Bag for ModeloFinalOutput {
    #[inline]
    fn is_empty(&self) -> bool {
        true && self.output_port.is_empty()
    }
    #[inline]
    fn clear(&mut self) {
        self.output_port.clear();
    }
}
pub struct ModeloFinalComponents<const W: usize> {
    generator: Generator,
    modelo_ho: HO<W>,
}
impl<const W: usize> ModeloFinalComponents<W> {
    #[inline]
    pub fn new(generator: Generator, modelo_ho: HO<W>) -> Self {
        Self {
            generator,
            modelo_ho,
        }
    }
}
#[doc = r" Wrapper struct holding mutable references to all inner components' inputs."]
pub struct ModeloFinalComponentsInput<'__xdevs_inner, const W: usize> {
    pub generator: <Generator as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
    pub modelo_ho: <HO<W> as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
}
#[doc = r" Wrapper struct holding references to all inner components' outputs."]
pub struct ModeloFinalComponentsOutput<'__xdevs_inner, const W: usize> {
    pub generator: <Generator as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
    pub modelo_ho: <HO<W> as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
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
    pub fn build(generator: Generator, modelo_ho: HO<W>) -> Self {
        Self {
            input: ModeloFinalInput::new(),
            output: ModeloFinalOutput::new(),
            t_last: 0.0,
            t_next: f64::INFINITY,
            components: ModeloFinalComponents::new(generator, modelo_ho),
        }
    }

    pub fn get_n_atomics(&self) -> usize {
        self.components.modelo_ho.get_n_atomics()
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
            xdevs::traits::AbstractSimulator::start(&mut self.components.modelo_ho, t_start),
        );
        xdevs::traits::Component::set_t_next(self, t_next);
        t_next
    }
    #[inline]
    fn stop(&mut self, t_stop: f64) {
        xdevs::traits::AbstractSimulator::stop(&mut self.components.generator, t_stop);
        xdevs::traits::AbstractSimulator::stop(&mut self.components.modelo_ho, t_stop);
        xdevs::traits::Component::set_t_last(self, t_stop);
        xdevs::traits::Component::set_t_next(self, f64::INFINITY);
    }
    #[inline]
    fn lambda(&mut self, t: f64) {
        if t >= xdevs::traits::Component::get_t_next(self) {
            xdevs::traits::AbstractSimulator::lambda(&mut self.components.generator, t);
            xdevs::traits::AbstractSimulator::lambda(&mut self.components.modelo_ho, t);
            let generator_output =
                xdevs::traits::Component::get_out_ports(&self.components.generator);
            let modelo_ho_output =
                xdevs::traits::Component::get_out_ports(&self.components.modelo_ho);
            let component_outputs: ModeloFinalComponentsOutput<'_, W> =
                ModeloFinalComponentsOutput {
                    generator: generator_output,
                    modelo_ho: modelo_ho_output,
                };
            <Self as xdevs::Coupled>::eoc(&component_outputs, &mut self.output);
        }
    }
    #[inline]
    fn delta(&mut self, t: f64) -> f64 {
        {
            let (generator_input, generator_output) =
                xdevs::traits::Component::get_ports(&mut self.components.generator);
            let (modelo_ho_input, modelo_ho_output) =
                xdevs::traits::Component::get_ports(&mut self.components.modelo_ho);
            let component_outputs: ModeloFinalComponentsOutput<'_, W> =
                ModeloFinalComponentsOutput {
                    generator: generator_output,
                    modelo_ho: modelo_ho_output,
                };
            let mut component_inputs: ModeloFinalComponentsInput<'_, W> =
                ModeloFinalComponentsInput {
                    generator: generator_input,
                    modelo_ho: modelo_ho_input,
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
            xdevs::traits::AbstractSimulator::delta(&mut self.components.modelo_ho, t),
        );
        xdevs::traits::Component::clear_output(self);
        xdevs::traits::Component::clear_input(self);
        xdevs::traits::Component::set_t_last(self, t);
        xdevs::traits::Component::set_t_next(self, t_next);
        t_next
    }
}

//Implementación manual del trato Coupled para ModeloFinal
impl<const W: usize> xdevs::Coupled for ModeloFinal<W> {
    fn ic(from: &Self::ComponentsOutput<'_>, to: &mut Self::ComponentsInput<'_>) {
        from.generator.out_job.couple(&mut to.modelo_ho.input_port);
    }
}
//Fin acoplado ModeloFinal que recibe los datos de Generator y los introduce en el puerto de entrada del modelo HO
