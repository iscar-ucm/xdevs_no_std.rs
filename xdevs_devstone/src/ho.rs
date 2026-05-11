// use crate::common::*;
// use xdevs::traits::{AbstractSimulator, Component};

// //Inicio del modelo acoplado CoupAtom que contiene un único atómico
// // #[xdevs::coupled2]
// // pub struct CoupAtom {
// //     #[input]
// //     input_port_1: xdevs::port::Port<usize, 1>,
// //     input_port_2: xdevs::port::Port<usize, 1>,
// //     #[output]
// //     output_port_1: xdevs::port::Port<usize, 1>,
// //     output_port_2: xdevs::port::Port<usize, 1>,
// //     #[components]
// //     coup_atomic: Atom,
// // }

// // Recursive expansion of coupled2 macro
// // ======================================
// pub struct CoupAtomComponents {
//     coup_atomic: Atom,
// }
// impl CoupAtomComponents {
//     #[inline]
//     pub fn new(coup_atomic: Atom) -> Self {
//         Self { coup_atomic }
//     }
// }
// #[doc = r" Wrapper struct holding mutable references to all inner components' inputs."]
// pub struct CoupAtomComponentsInput<'__xdevs_inner> {
//     pub coup_atomic: <Atom as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
// }
// #[doc = r" Wrapper struct holding references to all inner components' outputs."]
// pub struct CoupAtomComponentsOutput<'__xdevs_inner> {
//     pub coup_atomic: <Atom as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
// }
// pub struct CoupAtom {
//     pub input: CoupHOInput,
//     pub output: CoupHOOutput,
//     pub t_last: f64,
//     pub t_next: f64,
//     pub components: CoupAtomComponents,
// }
// impl CoupAtom {
//     #[inline]
//     pub fn build(coup_atomic: Atom) -> Self {
//         Self {
//             input: CoupHOInput::new(),
//             output: CoupHOOutput::new(),
//             t_last: 0.0,
//             t_next: f64::INFINITY,
//             components: CoupAtomComponents::new(coup_atomic),
//         }
//     }
// }
// unsafe impl xdevs::traits::Component for CoupAtom {
//     type Input = CoupHOInput;
//     type Output = CoupHOOutput;
//     type InputRef<'__xdevs_ports>
//         = &'__xdevs_ports mut CoupHOInput
//     where
//         Self: '__xdevs_ports;
//     type OutputRef<'__xdevs_ports>
//         = &'__xdevs_ports CoupHOOutput
//     where
//         Self: '__xdevs_ports;
//     #[inline]
//     fn get_t_last(&self) -> f64 {
//         self.t_last
//     }
//     #[inline]
//     fn set_t_last(&mut self, t_last: f64) {
//         self.t_last = t_last;
//     }
//     #[inline]
//     fn get_t_next(&self) -> f64 {
//         self.t_next
//     }
//     #[inline]
//     fn set_t_next(&mut self, t_next: f64) {
//         self.t_next = t_next;
//     }
//     #[inline]
//     fn get_input(&self) -> &Self::Input {
//         &self.input
//     }
//     #[inline]
//     fn get_input_mut(&mut self) -> &mut Self::Input {
//         &mut self.input
//     }
//     #[inline]
//     fn get_output(&self) -> &Self::Output {
//         &self.output
//     }
//     #[inline]
//     fn get_output_mut(&mut self) -> &mut Self::Output {
//         &mut self.output
//     }
//     #[inline]
//     fn get_ports(&mut self) -> (Self::InputRef<'_>, Self::OutputRef<'_>) {
//         (&mut self.input, &self.output)
//     }
//     #[inline]
//     fn get_out_ports(&self) -> Self::OutputRef<'_> {
//         &self.output
//     }
// }
// unsafe impl xdevs::traits::PartialCoupled for CoupAtom {
//     type ComponentsInput<'__xdevs_inner>
//         = CoupAtomComponentsInput<'__xdevs_inner>
//     where
//         Self: '__xdevs_inner;
//     type ComponentsOutput<'__xdevs_inner>
//         = CoupAtomComponentsOutput<'__xdevs_inner>
//     where
//         Self: '__xdevs_inner;
// }
// unsafe impl xdevs::traits::AbstractSimulator for CoupAtom {
//     #[inline]
//     fn start(&mut self, t_start: f64) -> f64 {
//         xdevs::traits::Component::set_t_last(self, t_start);
//         let mut t_next = f64::INFINITY;
//         t_next = f64::min(
//             t_next,
//             xdevs::traits::AbstractSimulator::start(&mut self.components.coup_atomic, t_start),
//         );
//         xdevs::traits::Component::set_t_next(self, t_next);
//         t_next
//     }
//     #[inline]
//     fn stop(&mut self, t_stop: f64) {
//         xdevs::traits::AbstractSimulator::stop(&mut self.components.coup_atomic, t_stop);
//         xdevs::traits::Component::set_t_last(self, t_stop);
//         xdevs::traits::Component::set_t_next(self, f64::INFINITY);
//     }
//     #[inline]
//     fn lambda(&mut self, t: f64) {
//         if t >= xdevs::traits::Component::get_t_next(self) {
//             xdevs::traits::AbstractSimulator::lambda(&mut self.components.coup_atomic, t);
//             let coup_atomic_output =
//                 xdevs::traits::Component::get_out_ports(&self.components.coup_atomic);
//             let component_outputs: CoupAtomComponentsOutput<'_> = CoupAtomComponentsOutput {
//                 coup_atomic: coup_atomic_output,
//             };
//             <Self as xdevs::Coupled>::eoc(&component_outputs, &mut self.output);
//         }
//     }
//     #[inline]
//     fn delta(&mut self, t: f64) -> f64 {
//         {
//             let (coup_atomic_input, coup_atomic_output) =
//                 xdevs::traits::Component::get_ports(&mut self.components.coup_atomic);
//             let component_outputs: CoupAtomComponentsOutput<'_> = CoupAtomComponentsOutput {
//                 coup_atomic: coup_atomic_output,
//             };
//             let mut component_inputs: CoupAtomComponentsInput<'_> = CoupAtomComponentsInput {
//                 coup_atomic: coup_atomic_input,
//             };
//             <Self as xdevs::Coupled>::eic(&self.input, &mut component_inputs);
//             <Self as xdevs::Coupled>::ic(&component_outputs, &mut component_inputs);
//         }
//         let mut t_next = f64::INFINITY;
//         t_next = f64::min(
//             t_next,
//             xdevs::traits::AbstractSimulator::delta(&mut self.components.coup_atomic, t),
//         );
//         xdevs::traits::Component::clear_output(self);
//         xdevs::traits::Component::clear_input(self);
//         xdevs::traits::Component::set_t_last(self, t);
//         xdevs::traits::Component::set_t_next(self, t_next);
//         t_next
//     }
// }
// //Implementación manual del trato Coupled para CoupAtom
// impl xdevs::Coupled for CoupAtom {
//     fn eic(from: &Self::Input, to: &mut Self::ComponentsInput<'_>) {
//         from.input_port_1
//             .couple(&mut to.coup_atomic.input_port)
//             .unwrap();
//         let port = &from.input_port_1;
//         if !port.is_empty() {
//             unsafe {
//                 N_EIC += 1;
//             }
//         }
//     }

//     fn eoc(from: &Self::ComponentsOutput<'_>, to: &mut Self::Output) {
//         from.coup_atomic
//             .output_port
//             .couple(&mut to.output_port_1)
//             .unwrap();
//         let port = &from.coup_atomic.output_port;
//         if !port.is_empty() {
//             unsafe {
//                 N_EOC += 1;
//             }
//         }
//     }
// }

// impl CoupAtom {
//     pub fn new() -> Self {
//         Self::build(Atom::new())
//     }

//     pub fn get_n_internals(&self) -> usize {
//         self.components.coup_atomic.get_n_internals()
//     }

//     pub fn get_n_externals(&self) -> usize {
//         self.components.coup_atomic.get_n_externals()
//     }

//     pub fn get_n_events(&self) -> usize {
//         self.components.coup_atomic.get_n_events()
//     }

//     pub fn get_n_atomics(&self) -> usize {
//         self.components.coup_atomic.get_n_atomics()
//     }
// }
// //Fin del modelo acoplado CoupAtom que contiene un único atómico

// //Inicio enum con las opciones que puede haber en el modelo HO
// /*
// Enum con las opciones que puede haber en el modelo:
// - Acoplado que contiene un único atómico (CoupD(CoupAtom))
// - Acoplado que contiene un array de atómicos y otro acoplado del mismo tipo (RestoCoup(CoupHO<W>))
// */
// pub enum HO<const W: usize> {
//     CoupD(CoupAtom),
//     RestoCoup(CoupHO<W>),
// }

// //Implementacion manual de AbstractSimulator para HO
// unsafe impl<const W: usize> AbstractSimulator for HO<W> {
//     fn start(&mut self, t_start: f64) -> f64 {
//         match self {
//             HO::CoupD(coup_d) => coup_d.start(t_start),
//             HO::RestoCoup(coup_r) => coup_r.start(t_start),
//         }
//     }

//     fn stop(&mut self, t_stop: f64) {
//         match self {
//             HO::CoupD(coup_d) => coup_d.stop(t_stop),
//             HO::RestoCoup(coup_r) => coup_r.stop(t_stop),
//         }
//     }

//     fn lambda(&mut self, t: f64) {
//         match self {
//             HO::CoupD(coup_d) => coup_d.lambda(t),
//             HO::RestoCoup(coup_r) => coup_r.lambda(t),
//         }
//     }

//     fn delta(&mut self, t: f64) -> f64 {
//         match self {
//             HO::CoupD(coup_d) => coup_d.delta(t),
//             HO::RestoCoup(coup_r) => coup_r.delta(t),
//         }
//     }
// }

// //Implementación manual de Component para HO
// unsafe impl<const W: usize> Component for HO<W> {
//     type Input = CoupHOInput;

//     type Output = CoupHOOutput<W>;

//     type InputRef<'a>
//         = &'a mut Self::Input
//     where
//         Self: 'a;
//     type OutputRef<'a>
//         = &'a Self::Output
//     where
//         Self: 'a;

//     fn get_t_last(&self) -> f64 {
//         match self {
//             HO::CoupD(coup_d) => coup_d.get_t_last(),
//             HO::RestoCoup(coup_r) => coup_r.get_t_last(),
//         }
//     }

//     fn set_t_last(&mut self, t_last: f64) {
//         match self {
//             HO::CoupD(coup_d) => coup_d.set_t_last(t_last),
//             HO::RestoCoup(coup_r) => coup_r.set_t_last(t_last),
//         }
//     }

//     fn get_t_next(&self) -> f64 {
//         match self {
//             HO::CoupD(coup_d) => coup_d.get_t_next(),
//             HO::RestoCoup(coup_r) => coup_r.get_t_next(),
//         }
//     }

//     fn set_t_next(&mut self, t_next: f64) {
//         match self {
//             HO::CoupD(coup_d) => coup_d.set_t_next(t_next),
//             HO::RestoCoup(coup_r) => coup_r.set_t_next(t_next),
//         }
//     }

//     /// Returns a reference to the model's input event bag.
//     fn get_input(&self) -> &Self::Input {
//         match self {
//             HO::CoupD(coup_d) => coup_d.get_input(),
//             HO::RestoCoup(coup_r) => coup_r.get_input(),
//         }
//     }

//     fn get_input_mut(&mut self) -> &mut Self::Input {
//         match self {
//             HO::CoupD(coup_d) => coup_d.get_input_mut(),
//             HO::RestoCoup(coup_r) => coup_r.get_input_mut(),
//         }
//     }

//     fn get_output(&self) -> &Self::Output {
//         match self {
//             HO::CoupD(coup_d) => coup_d.get_output(),
//             HO::RestoCoup(coup_r) => coup_r.get_output(),
//         }
//     }

//     fn get_output_mut(&mut self) -> &mut Self::Output {
//         match self {
//             HO::CoupD(coup_d) => coup_d.get_output_mut(),
//             HO::RestoCoup(coup_r) => coup_r.get_output_mut(),
//         }
//     }

//     fn get_ports(&mut self) -> (Self::InputRef<'_>, Self::OutputRef<'_>) {
//         match self {
//             HO::CoupD(coup_d) => coup_d.get_ports(),
//             HO::RestoCoup(coup_r) => coup_r.get_ports(),
//         }
//     }

//     fn get_out_ports(&self) -> Self::OutputRef<'_> {
//         match self {
//             HO::CoupD(coup_d) => coup_d.get_out_ports(),
//             HO::RestoCoup(coup_r) => coup_r.get_out_ports(),
//         }
//     }
// }

// impl<const W: usize> HO<W> {
//     pub fn get_n_internals(&self) -> usize {
//         match self {
//             HO::CoupD(coup_atom) => coup_atom.get_n_internals(),
//             HO::RestoCoup(coup_HO) => coup_HO.get_n_internals(),
//         }
//     }

//     pub fn get_n_externals(&self) -> usize {
//         match self {
//             HO::CoupD(coup_atom) => coup_atom.get_n_externals(),
//             HO::RestoCoup(coup_HO) => coup_HO.get_n_externals(),
//         }
//     }

//     pub fn get_n_events(&self) -> usize {
//         match self {
//             HO::CoupD(coup_atom) => coup_atom.get_n_events(),
//             HO::RestoCoup(coup_HO) => coup_HO.get_n_events(),
//         }
//     }

//     pub fn get_n_eic(&self) -> usize {
//         match self {
//             HO::CoupD(coup_atom) => get_n_eic(),
//             HO::RestoCoup(coup_HO) => get_n_eic(),
//         }
//     }

//     pub fn get_n_eoc(&self) -> usize {
//         match self {
//             HO::CoupD(coup_atom) => get_n_eoc(),
//             HO::RestoCoup(coup_HO) => get_n_eoc(),
//         }
//     }

//     pub fn get_n_ic(&self) -> usize {
//         match self {
//             HO::CoupD(coup_atom) => get_n_ic(),
//             HO::RestoCoup(coup_HO) => get_n_ic(),
//         }
//     }

//     pub fn get_n_atomics(&self) -> usize {
//         match self {
//             HO::CoupD(coup_atom) => coup_atom.get_n_atomics(),
//             HO::RestoCoup(coup_HO) => coup_HO.get_n_atomics(),
//         }
//     }
// }
// //Fin enum con las opciones que puede haber en el modelo HO

// //Inicio del acoplado con con un array de atómicos con 2 inputs, un atómico con un input y otro acoplado igual (el acoplado tiene 2 outputs)
// // #[xdevs::coupled2]
// // pub struct CoupHO<const W: usize> {
// //     #[input]
// //     input_port_1: xdevs::port::Port<usize, 1>,
// //     input_port_2: xdevs::port::Port<usize, 1>,
// //     #[output]
// //     output_port_1: xdevs::port::Port<usize, 1>,
// //     output_port_2: xdevs::port::Port<usize, W>,
// //     #[components]
// //     comp_atomic: [Atom2Inputs2Outputs; W],
// //     comp_coupled: Box<HO<W>>,
// // }

// // Recursive expansion of coupled2 macro
// // ======================================

// #[derive(Debug, Default)]
// pub struct CoupHOInput {
//     pub input_port_1: xdevs::port::Port<usize, 1>,
//     pub input_port_2: xdevs::port::Port<usize, 1>,
// }
// impl CoupHOInput {
//     #[inline]
//     pub const fn new() -> Self {
//         Self {
//             input_port_1: xdevs::port::Port::new(),
//             input_port_2: xdevs::port::Port::new(),
//         }
//     }
// }
// unsafe impl xdevs::traits::Bag for CoupHOInput {
//     #[inline]
//     fn is_empty(&self) -> bool {
//         true && self.input_port_1.is_empty() && self.input_port_2.is_empty()
//     }
//     #[inline]
//     fn clear(&mut self) {
//         self.input_port_1.clear();
//         self.input_port_2.clear();
//     }
// }
// #[derive(Debug, Default)]
// pub struct CoupHOOutput<const W: usize> {
//     pub output_port_1: xdevs::port::Port<usize, 1>,
//     pub output_port_2: xdevs::port::Port<usize, W>,
// }
// impl<const W: usize> CoupHOOutput<W> {
//     #[inline]
//     pub const fn new() -> Self {
//         Self {
//             output_port_1: xdevs::port::Port::new(),
//             output_port_2: xdevs::port::Port::new(),
//         }
//     }
// }
// unsafe impl<const W: usize> xdevs::traits::Bag for CoupHOOutput<W> {
//     #[inline]
//     fn is_empty(&self) -> bool {
//         true && self.output_port_1.is_empty() && self.output_port_2.is_empty()
//     }
//     #[inline]
//     fn clear(&mut self) {
//         self.output_port_1.clear();
//         self.output_port_2.clear();
//     }
// }
// pub struct CoupHOComponents<const W: usize> {
//     comp_atomic: [Atom2Inputs2Outputs; W],
//     comp_coupled: Box<HO<W>>,
// }
// impl<const W: usize> CoupHOComponents<W> {
//     #[inline]
//     pub fn new(comp_atomic: [Atom2Inputs2Outputs; W], comp_coupled: Box<HO<W>>) -> Self {
//         Self {
//             comp_atomic,
//             comp_coupled,
//         }
//     }

//     pub fn get_n_internals(&self) -> usize {
//         let mut sum_int = self.comp_coupled.get_n_internals();
//         for atomic in self.comp_atomic.iter() {
//             sum_int += atomic.get_n_internals();
//         }
//         sum_int
//     }

//     pub fn get_n_externals(&self) -> usize {
//         let mut sum_ext = self.comp_coupled.get_n_externals();
//         for atomic in self.comp_atomic.iter() {
//             sum_ext += atomic.get_n_externals();
//         }
//         sum_ext
//     }

//     pub fn get_n_events(&self) -> usize {
//         let mut sum_ev = self.comp_coupled.get_n_events();
//         for atomic in self.comp_atomic.iter() {
//             sum_ev += atomic.get_n_events();
//         }
//         sum_ev
//     }

//     pub fn get_n_atomics(&self) -> usize {
//         let mut sum_atomic = self.comp_coupled.get_n_atomics();
//         println!("Número de atómicos en el acoplado interno: {}", sum_atomic);
//         for atomic in self.comp_atomic.iter() {
//             sum_atomic += 1;
//         }
//         sum_atomic
//     }
// }
// #[doc = r" Wrapper struct holding mutable references to all inner components' inputs."]
// pub struct CoupHOComponentsInput<'__xdevs_inner, const W: usize> {
//     pub comp_atomic:
//         <[Atom2Inputs2Outputs; W] as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
//     pub comp_coupled: <Box<HO<W>> as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
// }
// #[doc = r" Wrapper struct holding references to all inner components' outputs."]
// pub struct CoupHOComponentsOutput<'__xdevs_inner, const W: usize> {
//     pub comp_atomic:
//         <[Atom2Inputs2Outputs; W] as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
//     pub comp_coupled: <Box<HO<W>> as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
// }
// pub struct CoupHO<const W: usize> {
//     pub input: CoupHOInput,
//     pub output: CoupHOOutput<W>,
//     pub t_last: f64,
//     pub t_next: f64,
//     pub components: CoupHOComponents<W>,
// }
// impl<const W: usize> CoupHO<W> {
//     #[inline]
//     pub fn build(comp_atomic: [Atom2Inputs2Outputs; W], comp_coupled: Box<HO<W>>) -> Self {
//         Self {
//             input: CoupHOInput::new(),
//             output: CoupHOOutput::new(),
//             t_last: 0.0,
//             t_next: f64::INFINITY,
//             components: CoupHOComponents::new(comp_atomic, comp_coupled),
//         }
//     }
// }
// unsafe impl<const W: usize> xdevs::traits::Component for CoupHO<W> {
//     type Input = CoupHOInput;
//     type Output = CoupHOOutput<W>;
//     type InputRef<'__xdevs_ports>
//         = &'__xdevs_ports mut CoupHOInput
//     where
//         Self: '__xdevs_ports;
//     type OutputRef<'__xdevs_ports>
//         = &'__xdevs_ports CoupHOOutput<W>
//     where
//         Self: '__xdevs_ports;
//     #[inline]
//     fn get_t_last(&self) -> f64 {
//         self.t_last
//     }
//     #[inline]
//     fn set_t_last(&mut self, t_last: f64) {
//         self.t_last = t_last;
//     }
//     #[inline]
//     fn get_t_next(&self) -> f64 {
//         self.t_next
//     }
//     #[inline]
//     fn set_t_next(&mut self, t_next: f64) {
//         self.t_next = t_next;
//     }
//     #[inline]
//     fn get_input(&self) -> &Self::Input {
//         &self.input
//     }
//     #[inline]
//     fn get_input_mut(&mut self) -> &mut Self::Input {
//         &mut self.input
//     }
//     #[inline]
//     fn get_output(&self) -> &Self::Output {
//         &self.output
//     }
//     #[inline]
//     fn get_output_mut(&mut self) -> &mut Self::Output {
//         &mut self.output
//     }
//     #[inline]
//     fn get_ports(&mut self) -> (Self::InputRef<'_>, Self::OutputRef<'_>) {
//         (&mut self.input, &self.output)
//     }
//     #[inline]
//     fn get_out_ports(&self) -> Self::OutputRef<'_> {
//         &self.output
//     }
// }
// unsafe impl<const W: usize> xdevs::traits::PartialCoupled for CoupHO<W> {
//     type ComponentsInput<'__xdevs_inner>
//         = CoupHOComponentsInput<'__xdevs_inner, W>
//     where
//         Self: '__xdevs_inner;
//     type ComponentsOutput<'__xdevs_inner>
//         = CoupHOComponentsOutput<'__xdevs_inner, W>
//     where
//         Self: '__xdevs_inner;
// }
// unsafe impl<const W: usize> xdevs::traits::AbstractSimulator for CoupHO<W> {
//     #[inline]
//     fn start(&mut self, t_start: f64) -> f64 {
//         xdevs::traits::Component::set_t_last(self, t_start);
//         let mut t_next = f64::INFINITY;
//         t_next = f64::min(
//             t_next,
//             xdevs::traits::AbstractSimulator::start(&mut self.components.comp_atomic, t_start),
//         );
//         t_next = f64::min(
//             t_next,
//             xdevs::traits::AbstractSimulator::start(&mut self.components.comp_coupled, t_start),
//         );
//         xdevs::traits::Component::set_t_next(self, t_next);
//         t_next
//     }
//     #[inline]
//     fn stop(&mut self, t_stop: f64) {
//         xdevs::traits::AbstractSimulator::stop(&mut self.components.comp_atomic, t_stop);
//         xdevs::traits::AbstractSimulator::stop(&mut self.components.comp_coupled, t_stop);
//         xdevs::traits::Component::set_t_last(self, t_stop);
//         xdevs::traits::Component::set_t_next(self, f64::INFINITY);
//     }
//     #[inline]
//     fn lambda(&mut self, t: f64) {
//         if t >= xdevs::traits::Component::get_t_next(self) {
//             xdevs::traits::AbstractSimulator::lambda(&mut self.components.comp_atomic, t);
//             xdevs::traits::AbstractSimulator::lambda(&mut self.components.comp_coupled, t);
//             let comp_atomic_output =
//                 xdevs::traits::Component::get_out_ports(&self.components.comp_atomic);
//             let comp_coupled_output =
//                 xdevs::traits::Component::get_out_ports(&self.components.comp_coupled);
//             let component_outputs: CoupHOComponentsOutput<'_, W> = CoupHOComponentsOutput {
//                 comp_atomic: comp_atomic_output,
//                 comp_coupled: comp_coupled_output,
//             };
//             <Self as xdevs::Coupled>::eoc(&component_outputs, &mut self.output);
//         }
//     }
//     #[inline]
//     fn delta(&mut self, t: f64) -> f64 {
//         {
//             let (comp_atomic_input, comp_atomic_output) =
//                 xdevs::traits::Component::get_ports(&mut self.components.comp_atomic);
//             let (comp_coupled_input, comp_coupled_output) =
//                 xdevs::traits::Component::get_ports(&mut self.components.comp_coupled);
//             let component_outputs: CoupHOComponentsOutput<'_, W> = CoupHOComponentsOutput {
//                 comp_atomic: comp_atomic_output,
//                 comp_coupled: comp_coupled_output,
//             };
//             let mut component_inputs: CoupHOComponentsInput<'_, W> = CoupHOComponentsInput {
//                 comp_atomic: comp_atomic_input,
//                 comp_coupled: comp_coupled_input,
//             };
//             <Self as xdevs::Coupled>::eic(&self.input, &mut component_inputs);
//             <Self as xdevs::Coupled>::ic(&component_outputs, &mut component_inputs);
//         }
//         let mut t_next = f64::INFINITY;
//         t_next = f64::min(
//             t_next,
//             xdevs::traits::AbstractSimulator::delta(&mut self.components.comp_atomic, t),
//         );
//         t_next = f64::min(
//             t_next,
//             xdevs::traits::AbstractSimulator::delta(&mut self.components.comp_coupled, t),
//         );
//         xdevs::traits::Component::clear_output(self);
//         xdevs::traits::Component::clear_input(self);
//         xdevs::traits::Component::set_t_last(self, t);
//         xdevs::traits::Component::set_t_next(self, t_next);
//         t_next
//     }
// }

// //Implementación manual del trato Coupled para CoupHO
// impl<const W: usize> xdevs::Coupled for CoupHO<W> {
//     fn eic(from: &Self::Input, to: &mut Self::ComponentsInput<'_>) {
//         from.input_port_1
//             .couple(&mut to.comp_coupled.input_port_1)
//             .unwrap();
//         let port1 = &from.input_port_1;
//         if !port1.is_empty() {
//             unsafe {
//                 N_EIC += 1;
//             }
//         }

//         from.input_port_2
//             .couple(&mut to.comp_coupled.input_port_2)
//             .unwrap();
//         let port2 = &from.input_port_1;
//         if !port2.is_empty() {
//             unsafe {
//                 N_EIC += 1;
//             }
//         }

//         for atom_ports in to.comp_atomic.iter_mut() {
//             from.input_port_2
//                 .couple(&mut atom_ports.input_port_2)
//                 .unwrap();
//             let port = &from.input_port_2;
//             if !port.is_empty() {
//                 unsafe {
//                     N_EIC += 1;
//                 }
//             }
//         }
//     }

//     fn eoc(from: &Self::ComponentsOutput<'_>, to: &mut Self::Output) {
//         from.comp_coupled
//             .output_port_1
//             .couple(&mut to.output_port_1)
//             .unwrap();
//         let port1 = &from.comp_coupled.output_port_1;
//         if !port1.is_empty() {
//             unsafe {
//                 N_EOC += 1;
//             }
//         }

//         for output_port in from.comp_atomic {
//             output_port
//                 .output_port_1
//                 .couple(&mut to.output_port_2)
//                 .unwrap();
//             let port2 = &output_port;
//             if !port2.output_port_1.is_empty() {
//                 unsafe {
//                     N_EOC += 1;
//                 }
//             }
//         }
//     }

//     fn ic(from: &Self::ComponentsOutput<'_>, to: &mut Self::ComponentsInput<'_>) {
//         if W > 1 {
//             for i in 0..(W - 2) {
//                 from.comp_atomic[i]
//                     .output_port_2
//                     .couple(&mut to.comp_atomic[i + 1].input_port_1)
//                     .unwrap();
//                 let port = &from.comp_atomic[i].output_port_2;
//                 if !port.is_empty() {
//                     unsafe {
//                         N_IC += 1;
//                     }
//                 }
//             }
//         } else {
//             unsafe {
//                 N_IC += 1;
//             }
//         }
//     }
// }
// //Inicio del acoplado con con un array de atómicos con 2 inputs, un atómico con un input y otro acoplado igual (el acoplado tiene 2 outputs)
