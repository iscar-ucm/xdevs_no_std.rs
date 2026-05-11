// use crate::common::*;
// use xdevs::traits::{AbstractSimulator, Component};

// //CAMBIOS RESPECTO A LI:
// /*
// - Enum Cuop<W> --> LI<W> (en este caso HI<W>)
// - ModCoupLI<w> --> CoupLI<W> (en este caso CoupHI<W>)
// */
// //Inicio del modelo acoplado CoupAtom que contiene un único atómico
// // #[xdevs::coupled2]
// // pub struct CoupAtom {
// //     #[input]
// //     input_port: xdevs::port::Port<usize, 1>,
// //     #[output]
// //     output_port: xdevs::port::Port<usize, 1>,
// //     #[components]
// //     coup_atomic: Atom,
// // }

// // Recursive expansion of coupled2 macro
// // ======================================

// #[derive(Debug, Default)]
// pub struct CoupAtomInput {
//     pub input_port: xdevs::port::Port<usize, 1>,
// }
// impl CoupAtomInput {
//     #[inline]
//     pub const fn new() -> Self {
//         Self {
//             input_port: xdevs::port::Port::new(),
//         }
//     }
// }
// unsafe impl xdevs::traits::Bag for CoupAtomInput {
//     #[inline]
//     fn is_empty(&self) -> bool {
//         true && self.input_port.is_empty()
//     }
//     #[inline]
//     fn clear(&mut self) {
//         self.input_port.clear();
//     }
// }
// #[derive(Debug, Default)]
// pub struct CoupAtomOutput {
//     pub output_port: xdevs::port::Port<usize, 1>,
// }
// impl CoupAtomOutput {
//     #[inline]
//     pub const fn new() -> Self {
//         Self {
//             output_port: xdevs::port::Port::new(),
//         }
//     }
// }
// unsafe impl xdevs::traits::Bag for CoupAtomOutput {
//     #[inline]
//     fn is_empty(&self) -> bool {
//         true && self.output_port.is_empty()
//     }
//     #[inline]
//     fn clear(&mut self) {
//         self.output_port.clear();
//     }
// }
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
//     pub input: CoupAtomInput,
//     pub output: CoupAtomOutput,
//     pub t_last: f64,
//     pub t_next: f64,
//     pub components: CoupAtomComponents,
// }
// impl CoupAtom {
//     #[inline]
//     pub fn build(coup_atomic: Atom) -> Self {
//         Self {
//             input: CoupAtomInput::new(),
//             output: CoupAtomOutput::new(),
//             t_last: 0.0,
//             t_next: f64::INFINITY,
//             components: CoupAtomComponents::new(coup_atomic),
//         }
//     }
// }
// unsafe impl xdevs::traits::Component for CoupAtom {
//     type Input = CoupAtomInput;
//     type Output = CoupAtomOutput;
//     type InputRef<'__xdevs_ports>
//         = &'__xdevs_ports mut CoupAtomInput
//     where
//         Self: '__xdevs_ports;
//     type OutputRef<'__xdevs_ports>
//         = &'__xdevs_ports CoupAtomOutput
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
//         from.input_port.couple(&mut to.coup_atomic.input_port);
//         let port = &from.input_port;
//         if port.is_empty() {
//             unsafe {
//                 N_EIC_HI += 1;
//             }
//         }
//     }

//     fn eoc(from: &Self::ComponentsOutput<'_>, to: &mut Self::Output) {
//         from.coup_atomic.output_port.couple(&mut to.output_port);
//         let port = &from.coup_atomic.output_port;
//         if !port.is_empty() {
//             unsafe {
//                 N_EOC_HI += 1;
//             }
//         }
//     }

//     fn ic(from: &Self::ComponentsOutput<'_>, to: &mut Self::ComponentsInput<'_>) {
//         from.coup_atomic
//             .output_port
//             .couple(&mut to.coup_atomic.input_port);
//         let port = &from.coup_atomic.output_port;
//         if !port.is_empty() {
//             unsafe {
//                 N_IC_HI += 1;
//             }
//         }
//     }
// }
// //Fin del modelo acoplado CoupAtom que contiene un único atómico

// //Inicio enum con las opciones que puede haber en el modelo HI
// /*
// Enum con las opciones que puede haber en el modelo:
// - Acoplado que contiene un único atómico (CoupD(CoupAtom))
// - Acoplado que contiene un array de atómicos y otro acoplado del mismo tipo (RestoCoup(CoupHI<W>))
// */
// enum HI<const W: usize> {
//     CoupD(CoupAtom),
//     CoupW2(CoupHIW2),
//     CoupW3(CoupHIW3<W>),
// }

// //Implementacion manual de AbstractSimulator para HI
// unsafe impl<const W: usize> AbstractSimulator for HI<W> {
//     fn start(&mut self, t_start: f64) -> f64 {
//         match self {
//             HI::CoupD(coup_d) => coup_d.start(t_start),
//             HI::CoupW2(coup_w2) => coup_w2.start(t_start),
//             HI::CoupW3(coup_w3) => coup_w3.start(t_start),
//         }
//     }

//     fn stop(&mut self, t_stop: f64) {
//         match self {
//             HI::CoupD(coup_d) => coup_d.stop(t_stop),
//             HI::CoupW2(coup_w2) => coup_w2.stop(t_stop),
//             HI::CoupW3(coup_w3) => coup_w3.stop(t_stop),
//         }
//     }

//     fn lambda(&mut self, t: f64) {
//         match self {
//             HI::CoupD(coup_d) => coup_d.lambda(t),
//             HI::CoupW2(coup_w2) => coup_w2.lambda(t),
//             HI::CoupW3(coup_w3) => coup_w3.lambda(t),
//         }
//     }

//     fn delta(&mut self, t: f64) -> f64 {
//         match self {
//             HI::CoupD(coup_d) => coup_d.delta(t),
//             HI::CoupW2(coup_w2) => coup_w2.delta(t),
//             HI::CoupW3(coup_w3) => coup_w3.delta(t),
//         }
//     }
// }

// //Implementación manual de Component para HI
// unsafe impl<const W: usize> Component for HI<W> {
//     type Input = CoupAtomInput;

//     type Output = CoupAtomOutput;

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
//             HI::CoupD(coup_d) => coup_d.get_t_last(),
//             HI::CoupW2(coup_w2) => coup_w2.get_t_last(),
//             HI::CoupW3(coup_w3) => coup_w3.get_t_last(),
//         }
//     }

//     fn set_t_last(&mut self, t_last: f64) {
//         match self {
//             HI::CoupD(coup_d) => coup_d.set_t_last(t_last),
//             HI::CoupW2(coup_w2) => coup_w2.set_t_last(t_last),
//             HI::CoupW3(coup_w3) => coup_w3.set_t_last(t_last),
//         }
//     }

//     fn get_t_next(&self) -> f64 {
//         match self {
//             HI::CoupD(coup_d) => coup_d.get_t_next(),
//             HI::CoupW2(coup_w2) => coup_w2.get_t_next(),
//             HI::CoupW3(coup_w3) => coup_w3.get_t_next(),
//         }
//     }

//     fn set_t_next(&mut self, t_next: f64) {
//         match self {
//             HI::CoupD(coup_d) => coup_d.set_t_next(t_next),
//             HI::CoupW2(coup_w2) => coup_w2.set_t_next(t_next),
//             HI::CoupW3(coup_w3) => coup_w3.set_t_next(t_next),
//         }
//     }

//     /// Returns a reference to the model's input event bag.
//     fn get_input(&self) -> &Self::Input {
//         match self {
//             HI::CoupD(coup_d) => coup_d.get_input(),
//             HI::CoupW2(coup_w2) => coup_w2.get_input(),
//             HI::CoupW3(coup_w3) => coup_w3.get_input(),
//         }
//     }

//     fn get_input_mut(&mut self) -> &mut Self::Input {
//         match self {
//             HI::CoupD(coup_d) => coup_d.get_input_mut(),
//             HI::CoupW2(coup_w2) => coup_w2.get_input_mut(),
//             HI::CoupW3(coup_w3) => coup_w3.get_input_mut(),
//         }
//     }

//     fn get_output(&self) -> &Self::Output {
//         match self {
//             HI::CoupD(coup_d) => coup_d.get_output(),
//             HI::CoupW2(coup_w2) => coup_w2.get_output(),
//             HI::CoupW3(coup_w3) => coup_w3.get_output(),
//         }
//     }

//     fn get_output_mut(&mut self) -> &mut Self::Output {
//         match self {
//             HI::CoupD(coup_d) => coup_d.get_output_mut(),
//             HI::CoupW2(coup_w2) => coup_w2.get_output_mut(),
//             HI::CoupW3(coup_w3) => coup_w3.get_output_mut(),
//         }
//     }

//     fn get_ports(&mut self) -> (Self::InputRef<'_>, Self::OutputRef<'_>) {
//         match self {
//             HI::CoupD(coup_d) => coup_d.get_ports(),
//             HI::CoupW2(coup_w2) => coup_w2.get_ports(),
//             HI::CoupW3(coup_w3) => coup_w3.get_ports(),
//         }
//     }

//     fn get_out_ports(&self) -> Self::OutputRef<'_> {
//         match self {
//             HI::CoupD(coup_d) => coup_d.get_out_ports(),
//             HI::CoupW2(coup_w2) => coup_w2.get_out_ports(),
//             HI::CoupW3(coup_w3) => coup_w3.get_out_ports(),
//         }
//     }
// }

// //Fin enum con las opciones que puede haber en el modelo HI

// //Inicio del acoplado con con un atómico con un input y otro acoplado igual
// // #[xdevs::coupled2]
// // pub struct CoupHIW2 {
// //     #[input]
// //     input_port: xdevs::port::Port<usize, 1>,
// //     #[output]
// //     output_port: xdevs::port::Port<usize, 1>,
// //     #[components]
// //     coup_atom_w2: Atom,
// //     coup_coupled: Box<HI<1>>, //Siempre W = 1 porque W=WIDTH-1 y estamos hablando del caso de un acoplado y un único atómico
// // }
// // Recursive expansion of coupled2 macro
// // ======================================

// #[derive(Debug, Default)]
// pub struct CoupHIW2Input {
//     pub input_port: xdevs::port::Port<usize, 1>,
// }
// impl CoupHIW2Input {
//     #[inline]
//     pub const fn new() -> Self {
//         Self {
//             input_port: xdevs::port::Port::new(),
//         }
//     }
// }
// unsafe impl xdevs::traits::Bag for CoupHIW2Input {
//     #[inline]
//     fn is_empty(&self) -> bool {
//         true && self.input_port.is_empty()
//     }
//     #[inline]
//     fn clear(&mut self) {
//         self.input_port.clear();
//     }
// }
// #[derive(Debug, Default)]
// pub struct CoupHIW2Output {
//     pub output_port: xdevs::port::Port<usize, 1>,
// }
// impl CoupHIW2Output {
//     #[inline]
//     pub const fn new() -> Self {
//         Self {
//             output_port: xdevs::port::Port::new(),
//         }
//     }
// }
// unsafe impl xdevs::traits::Bag for CoupHIW2Output {
//     #[inline]
//     fn is_empty(&self) -> bool {
//         true && self.output_port.is_empty()
//     }
//     #[inline]
//     fn clear(&mut self) {
//         self.output_port.clear();
//     }
// }
// pub struct CoupHIW2Components {
//     coup_atom_w2: Atom,
//     coup_coupled: Box<HI<1>>,
// }
// impl CoupHIW2Components {
//     #[inline]
//     pub fn new(coup_atom_w2: Atom, coup_coupled: Box<HI<1>>) -> Self {
//         Self {
//             coup_atom_w2,
//             coup_coupled,
//         }
//     }
// }
// #[doc = r" Wrapper struct holding mutable references to all inner components' inputs."]
// pub struct CoupHIW2ComponentsInput<'__xdevs_inner> {
//     pub coup_atom_w2: <Atom as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
//     pub coup_coupled: <Box<HI<1>> as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
// }
// #[doc = r" Wrapper struct holding references to all inner components' outputs."]
// pub struct CoupHIW2ComponentsOutput<'__xdevs_inner> {
//     pub coup_atom_w2: <Atom as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
//     pub coup_coupled: <Box<HI<1>> as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
// }
// pub struct CoupHIW2 {
//     pub input: CoupHIW2Input,
//     pub output: CoupHIW2Output,
//     pub t_last: f64,
//     pub t_next: f64,
//     pub components: CoupHIW2Components,
// }
// impl CoupHIW2 {
//     #[inline]
//     pub fn build(coup_atom_w2: Atom, coup_coupled: Box<HI<1>>) -> Self {
//         Self {
//             input: CoupHIW2Input::new(),
//             output: CoupHIW2Output::new(),
//             t_last: 0.0,
//             t_next: f64::INFINITY,
//             components: CoupHIW2Components::new(coup_atom_w2, coup_coupled),
//         }
//     }
// }
// unsafe impl xdevs::traits::Component for CoupHIW2 {
//     type Input = CoupHIW2Input;
//     type Output = CoupHIW2Output;
//     type InputRef<'__xdevs_ports>
//         = &'__xdevs_ports mut CoupHIW2Input
//     where
//         Self: '__xdevs_ports;
//     type OutputRef<'__xdevs_ports>
//         = &'__xdevs_ports CoupHIW2Output
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
// unsafe impl xdevs::traits::PartialCoupled for CoupHIW2 {
//     type ComponentsInput<'__xdevs_inner>
//         = CoupHIW2ComponentsInput<'__xdevs_inner>
//     where
//         Self: '__xdevs_inner;
//     type ComponentsOutput<'__xdevs_inner>
//         = CoupHIW2ComponentsOutput<'__xdevs_inner>
//     where
//         Self: '__xdevs_inner;
// }
// unsafe impl xdevs::traits::AbstractSimulator for CoupHIW2 {
//     #[inline]
//     fn start(&mut self, t_start: f64) -> f64 {
//         xdevs::traits::Component::set_t_last(self, t_start);
//         let mut t_next = f64::INFINITY;
//         t_next = f64::min(
//             t_next,
//             xdevs::traits::AbstractSimulator::start(&mut self.components.coup_atom_w2, t_start),
//         );
//         t_next = f64::min(
//             t_next,
//             xdevs::traits::AbstractSimulator::start(&mut self.components.coup_coupled, t_start),
//         );
//         xdevs::traits::Component::set_t_next(self, t_next);
//         t_next
//     }
//     #[inline]
//     fn stop(&mut self, t_stop: f64) {
//         xdevs::traits::AbstractSimulator::stop(&mut self.components.coup_atom_w2, t_stop);
//         xdevs::traits::AbstractSimulator::stop(&mut self.components.coup_coupled, t_stop);
//         xdevs::traits::Component::set_t_last(self, t_stop);
//         xdevs::traits::Component::set_t_next(self, f64::INFINITY);
//     }
//     #[inline]
//     fn lambda(&mut self, t: f64) {
//         if t >= xdevs::traits::Component::get_t_next(self) {
//             xdevs::traits::AbstractSimulator::lambda(&mut self.components.coup_atom_w2, t);
//             xdevs::traits::AbstractSimulator::lambda(&mut self.components.coup_coupled, t);
//             let coup_atom_w2_output =
//                 xdevs::traits::Component::get_out_ports(&self.components.coup_atom_w2);
//             let coup_coupled_output =
//                 xdevs::traits::Component::get_out_ports(&self.components.coup_coupled);
//             let component_outputs: CoupHIW2ComponentsOutput<'_> = CoupHIW2ComponentsOutput {
//                 coup_atom_w2: coup_atom_w2_output,
//                 coup_coupled: coup_coupled_output,
//             };
//             <Self as xdevs::Coupled>::eoc(&component_outputs, &mut self.output);
//         }
//     }
//     #[inline]
//     fn delta(&mut self, t: f64) -> f64 {
//         {
//             let (coup_atom_w2_input, coup_atom_w2_output) =
//                 xdevs::traits::Component::get_ports(&mut self.components.coup_atom_w2);
//             let (coup_coupled_input, coup_coupled_output) =
//                 xdevs::traits::Component::get_ports(&mut self.components.coup_coupled);
//             let component_outputs: CoupHIW2ComponentsOutput<'_> = CoupHIW2ComponentsOutput {
//                 coup_atom_w2: coup_atom_w2_output,
//                 coup_coupled: coup_coupled_output,
//             };
//             let mut component_inputs: CoupHIW2ComponentsInput<'_> = CoupHIW2ComponentsInput {
//                 coup_atom_w2: coup_atom_w2_input,
//                 coup_coupled: coup_coupled_input,
//             };
//             <Self as xdevs::Coupled>::eic(&self.input, &mut component_inputs);
//             <Self as xdevs::Coupled>::ic(&component_outputs, &mut component_inputs);
//         }
//         let mut t_next = f64::INFINITY;
//         t_next = f64::min(
//             t_next,
//             xdevs::traits::AbstractSimulator::delta(&mut self.components.coup_atom_w2, t),
//         );
//         t_next = f64::min(
//             t_next,
//             xdevs::traits::AbstractSimulator::delta(&mut self.components.coup_coupled, t),
//         );
//         xdevs::traits::Component::clear_output(self);
//         xdevs::traits::Component::clear_input(self);
//         xdevs::traits::Component::set_t_last(self, t);
//         xdevs::traits::Component::set_t_next(self, t_next);
//         t_next
//     }
// }

// //Implementación manual del trato Coupled para CoupHIW2
// impl xdevs::Coupled for CoupHIW2 {
//     fn eic(from: &Self::Input, to: &mut Self::ComponentsInput<'_>) {
//         from.input_port.couple(&mut to.coup_atom_w2.input_port);
//         from.input_port.couple(&mut to.coup_coupled.input_port);
//         let port = &from.input_port;
//         if !port.is_empty() {
//             unsafe {
//                 N_EIC_HI += 1;
//             }
//         }
//     }

//     fn eoc(from: &Self::ComponentsOutput<'_>, to: &mut Self::Output) {
//         from.coup_coupled.output_port.couple(&mut to.output_port);
//         let port = &from.coup_coupled.output_port;
//         if !port.is_empty() {
//             unsafe {
//                 N_EOC_HI += 1;
//             }
//         }
//     }

//     fn ic(from: &Self::ComponentsOutput<'_>, to: &mut Self::ComponentsInput<'_>) {} //Como tenemos width=2, no hay ics
// }
// //Fin del acoplado con con un atómico con un input y otro acoplado igual

// //Inicio del acoplado con con un array de atómicos con 2 inputs, un atómico con un input y otro acoplado igual
// // #[xdevs::coupled2]
// // pub struct CoupHIW3<const W: usize> {
// //     #[input]
// //     input_port: xdevs::port::Port<usize, 1>,
// //     #[output]
// //     output_port: xdevs::port::Port<usize, 1>,
// //     #[components]
// //     coup_atom_1_port: Atom,
// //     coup_atom_2_ports: [Atom2Inputs; W],
// //     coup_coupled: Box<HI<W>>,
// // }

// // Recursive expansion of coupled2 macro
// // ======================================

// #[derive(Debug, Default)]
// pub struct CoupHIW3Input {
//     pub input_port: xdevs::port::Port<usize, 1>,
// }
// impl CoupHIW3Input {
//     #[inline]
//     pub const fn new() -> Self {
//         Self {
//             input_port: xdevs::port::Port::new(),
//         }
//     }
// }
// unsafe impl xdevs::traits::Bag for CoupHIW3Input {
//     #[inline]
//     fn is_empty(&self) -> bool {
//         true && self.input_port.is_empty()
//     }
//     #[inline]
//     fn clear(&mut self) {
//         self.input_port.clear();
//     }
// }
// #[derive(Debug, Default)]
// pub struct CoupHIW3Output {
//     pub output_port: xdevs::port::Port<usize, 1>,
// }
// impl CoupHIW3Output {
//     #[inline]
//     pub const fn new() -> Self {
//         Self {
//             output_port: xdevs::port::Port::new(),
//         }
//     }
// }
// unsafe impl xdevs::traits::Bag for CoupHIW3Output {
//     #[inline]
//     fn is_empty(&self) -> bool {
//         true && self.output_port.is_empty()
//     }
//     #[inline]
//     fn clear(&mut self) {
//         self.output_port.clear();
//     }
// }
// pub struct CoupHIW3Components<const W: usize> {
//     coup_atom_1_port: Atom,
//     coup_atom_2_ports: [Atom2Inputs; W],
//     coup_coupled: Box<HI<W>>,
// }
// impl<const W: usize> CoupHIW3Components<W> {
//     #[inline]
//     pub fn new(
//         coup_atom_1_port: Atom,
//         coup_atom_2_ports: [Atom2Inputs; W],
//         coup_coupled: Box<HI<W>>,
//     ) -> Self {
//         Self {
//             coup_atom_1_port,
//             coup_atom_2_ports,
//             coup_coupled,
//         }
//     }
// }
// #[doc = r" Wrapper struct holding mutable references to all inner components' inputs."]
// pub struct CoupHIW3ComponentsInput<'__xdevs_inner, const W: usize> {
//     pub coup_atom_1_port: <Atom as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
//     pub coup_atom_2_ports: <[Atom2Inputs; W] as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
//     pub coup_coupled: <Box<HI<W>> as xdevs::traits::Component>::InputRef<'__xdevs_inner>,
// }
// #[doc = r" Wrapper struct holding references to all inner components' outputs."]
// pub struct CoupHIW3ComponentsOutput<'__xdevs_inner, const W: usize> {
//     pub coup_atom_1_port: <Atom as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
//     pub coup_atom_2_ports:
//         <[Atom2Inputs; W] as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
//     pub coup_coupled: <Box<HI<W>> as xdevs::traits::Component>::OutputRef<'__xdevs_inner>,
// }
// pub struct CoupHIW3<const W: usize> {
//     pub input: CoupHIW3Input,
//     pub output: CoupHIW3Output,
//     pub t_last: f64,
//     pub t_next: f64,
//     pub components: CoupHIW3Components<W>,
// }
// impl<const W: usize> CoupHIW3<W> {
//     #[inline]
//     pub fn build(
//         coup_atom_1_port: Atom,
//         coup_atom_2_ports: [Atom2Inputs; W],
//         coup_coupled: Box<HI<W>>,
//     ) -> Self {
//         Self {
//             input: CoupHIW3Input::new(),
//             output: CoupHIW3Output::new(),
//             t_last: 0.0,
//             t_next: f64::INFINITY,
//             components: CoupHIW3Components::new(coup_atom_1_port, coup_atom_2_ports, coup_coupled),
//         }
//     }
// }
// unsafe impl<const W: usize> xdevs::traits::Component for CoupHIW3<W> {
//     type Input = CoupHIW3Input;
//     type Output = CoupHIW3Output;
//     type InputRef<'__xdevs_ports>
//         = &'__xdevs_ports mut CoupHIW3Input
//     where
//         Self: '__xdevs_ports;
//     type OutputRef<'__xdevs_ports>
//         = &'__xdevs_ports CoupHIW3Output
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
// unsafe impl<const W: usize> xdevs::traits::PartialCoupled for CoupHIW3<W> {
//     type ComponentsInput<'__xdevs_inner>
//         = CoupHIW3ComponentsInput<'__xdevs_inner, W>
//     where
//         Self: '__xdevs_inner;
//     type ComponentsOutput<'__xdevs_inner>
//         = CoupHIW3ComponentsOutput<'__xdevs_inner, W>
//     where
//         Self: '__xdevs_inner;
// }
// unsafe impl<const W: usize> xdevs::traits::AbstractSimulator for CoupHIW3<W> {
//     #[inline]
//     fn start(&mut self, t_start: f64) -> f64 {
//         xdevs::traits::Component::set_t_last(self, t_start);
//         let mut t_next = f64::INFINITY;
//         t_next = f64::min(
//             t_next,
//             xdevs::traits::AbstractSimulator::start(&mut self.components.coup_atom_1_port, t_start),
//         );
//         t_next = f64::min(
//             t_next,
//             xdevs::traits::AbstractSimulator::start(
//                 &mut self.components.coup_atom_2_ports,
//                 t_start,
//             ),
//         );
//         t_next = f64::min(
//             t_next,
//             xdevs::traits::AbstractSimulator::start(&mut self.components.coup_coupled, t_start),
//         );
//         xdevs::traits::Component::set_t_next(self, t_next);
//         t_next
//     }
//     #[inline]
//     fn stop(&mut self, t_stop: f64) {
//         xdevs::traits::AbstractSimulator::stop(&mut self.components.coup_atom_1_port, t_stop);
//         xdevs::traits::AbstractSimulator::stop(&mut self.components.coup_atom_2_ports, t_stop);
//         xdevs::traits::AbstractSimulator::stop(&mut self.components.coup_coupled, t_stop);
//         xdevs::traits::Component::set_t_last(self, t_stop);
//         xdevs::traits::Component::set_t_next(self, f64::INFINITY);
//     }
//     #[inline]
//     fn lambda(&mut self, t: f64) {
//         if t >= xdevs::traits::Component::get_t_next(self) {
//             xdevs::traits::AbstractSimulator::lambda(&mut self.components.coup_atom_1_port, t);
//             xdevs::traits::AbstractSimulator::lambda(&mut self.components.coup_atom_2_ports, t);
//             xdevs::traits::AbstractSimulator::lambda(&mut self.components.coup_coupled, t);
//             let coup_atom_1_port_output =
//                 xdevs::traits::Component::get_out_ports(&self.components.coup_atom_1_port);
//             let coup_atom_2_ports_output =
//                 xdevs::traits::Component::get_out_ports(&self.components.coup_atom_2_ports);
//             let coup_coupled_output =
//                 xdevs::traits::Component::get_out_ports(&self.components.coup_coupled);
//             let component_outputs: CoupHIW3ComponentsOutput<'_, W> = CoupHIW3ComponentsOutput {
//                 coup_atom_1_port: coup_atom_1_port_output,
//                 coup_atom_2_ports: coup_atom_2_ports_output,
//                 coup_coupled: coup_coupled_output,
//             };
//             <Self as xdevs::Coupled>::eoc(&component_outputs, &mut self.output);
//         }
//     }
//     #[inline]
//     fn delta(&mut self, t: f64) -> f64 {
//         {
//             let (coup_atom_1_port_input, coup_atom_1_port_output) =
//                 xdevs::traits::Component::get_ports(&mut self.components.coup_atom_1_port);
//             let (coup_atom_2_ports_input, coup_atom_2_ports_output) =
//                 xdevs::traits::Component::get_ports(&mut self.components.coup_atom_2_ports);
//             let (coup_coupled_input, coup_coupled_output) =
//                 xdevs::traits::Component::get_ports(&mut self.components.coup_coupled);
//             let component_outputs: CoupHIW3ComponentsOutput<'_, W> = CoupHIW3ComponentsOutput {
//                 coup_atom_1_port: coup_atom_1_port_output,
//                 coup_atom_2_ports: coup_atom_2_ports_output,
//                 coup_coupled: coup_coupled_output,
//             };
//             let mut component_inputs: CoupHIW3ComponentsInput<'_, W> = CoupHIW3ComponentsInput {
//                 coup_atom_1_port: coup_atom_1_port_input,
//                 coup_atom_2_ports: coup_atom_2_ports_input,
//                 coup_coupled: coup_coupled_input,
//             };
//             <Self as xdevs::Coupled>::eic(&self.input, &mut component_inputs);
//             <Self as xdevs::Coupled>::ic(&component_outputs, &mut component_inputs);
//         }
//         let mut t_next = f64::INFINITY;
//         t_next = f64::min(
//             t_next,
//             xdevs::traits::AbstractSimulator::delta(&mut self.components.coup_atom_1_port, t),
//         );
//         t_next = f64::min(
//             t_next,
//             xdevs::traits::AbstractSimulator::delta(&mut self.components.coup_atom_2_ports, t),
//         );
//         t_next = f64::min(
//             t_next,
//             xdevs::traits::AbstractSimulator::delta(&mut self.components.coup_coupled, t),
//         );
//         xdevs::traits::Component::clear_output(self);
//         xdevs::traits::Component::clear_input(self);
//         xdevs::traits::Component::set_t_last(self, t);
//         xdevs::traits::Component::set_t_next(self, t_next);
//         t_next
//     }
// }

// //Implementación manual del trato Coupled para CoupHIW3
// impl<const W: usize> xdevs::Coupled for CoupHIW3<W>{
//     fn eic(from: &Self::Input, to: &mut Self::ComponentsInput<'_>) {
//         for
//     }
// }

// //Fin del acoplado con con un array de atómicos con 2 inputs, un atómico con un input y otro acoplado igual
