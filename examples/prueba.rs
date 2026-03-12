use xdevs::traits::{AbstractSimulator, Component};

//Atomic model
pub mod atom {
    #[xdevs::atomic]
    pub struct Atom {
        #[input]
        atom_input_port: xdevs::port::Port<bool, 1>,
        #[output]
        atom_output_port: xdevs::port::Port<usize, 1>,
        #[state]
        sigma: f64,
        period: f64,
        count: usize,
    }

    impl xdevs::Atomic for Atom {
        fn delta_int(state: &mut Self::State) {
            state.count += 1;
            state.sigma = state.period;
        }

        fn lambda(state: &Self::State, output: &mut Self::Output) {
            output.atom_output_port.add_value(state.count).unwrap();
        }

        fn ta(state: &Self::State) -> f64 {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, elapsed: f64, input: &Self::Input) {
            state.sigma -= elapsed;
            if let Some(&stop) = input.atom_input_port.get_values().last() {
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
}

/*ENUM: hay 2 opciones
- Opción 1: acoplado con un único atomic
- Opción 2: acoplado con un atomic y un acoplado, y el acoplado a su vez con un atomic y un acoplado, etc. (estructura recursiva)
*/

//Coupled model
pub mod mod_coup {
    //falta la entrada y la salida del modelo
    //después volver a desenrollar la macro y volver a quitar los const
    use super::atom::Atom;

    pub enum Coup<const W: usize, const P: usize> {
        CoupD(Atom), //en vez de Atom, Coup (como no es recursivo no hace falta pasarlo por referencia)
        RestoCoup(Box<Self>),
    }

    impl<const W: usize, const P: usize> Coup<W, P> {
        fn new(&self) {
            match self {
                Coup::CoupD(atom) => {}
                Coup::RestoCoup(coup) => {}
            }
        }
    }

    pub struct ModCoup<const W: usize, const P: usize> {
        #[components]
        //constante con la width y otra que sea la width-1 para los genericos
        comp_atomic: [Atom; W], //Coup::CoupD(Atom::new(period: f64)),
        comp_coupled: Box<Coup<W, P>>, //Coup::RestoCoup(Box::new(Coup::CoupD(Atom::new(period: f64)))),
    }

    /*
        // Recursive expansion of coupled2 macro
    // ======================================

    #[derive(Debug,Default)]
    pub struct ModCoupInput{}

    impl ModCoupInput {
        #[inline]
        pub const fn new() -> Self {
            Self{}

        }


    unsafe impl xdevs::traits::Bag for ModCoupInput {
        #[inline]
        fn is_empty(&self) -> bool {
            true
        }
        #[inline]
        fn clear(&mut self){}


        }
    #[derive(Debug,Default)]
    pub struct ModCoupOutput{}

    impl ModCoupOutput {
        #[inline]
        pub const fn new() -> Self {
            Self{}

        }

        }
    unsafe impl xdevs::traits::Bag for ModCoupOutput {
        #[inline]
        fn is_empty(&self) -> bool {
            true
        }
        #[inline]
        fn clear(&mut self){}


        }
    pub struct ModCoupComponents<const W:usize,const P:usize>{
        comp_atomic:[Atom;
        W],comp_coupled:Box<Coup<W,P> > ,
    }
    impl <const W:usize,const P:usize>ModCoupComponents<W,P>{
        #[inline]
        pub fn new(comp_atomic:[Atom;
        W],comp_coupled:Box<Coup<W,P> >) -> Self {
            Self {
                comp_atomic,comp_coupled
            }
        }

        }
    #[doc = r" Wrapper struct holding mutable references to all inner components' inputs."]
    pub struct ModCoupComponentsInput<'__xdevs_inner,const W:usize,const P:usize>{
        pub comp_atomic: <[Atom;
        W]as xdevs::traits::Component> ::InputRef<'__xdevs_inner> ,pub comp_coupled: <Box<Coup<W,P> >as xdevs::traits::Component> ::InputRef<'__xdevs_inner>
    }
    #[doc = r" Wrapper struct holding references to all inner components' outputs."]
    pub struct ModCoupComponentsOutput<'__xdevs_inner,const W:usize,const P:usize>{
        pub comp_atomic: <[Atom;
        W]as xdevs::traits::Component> ::OutputRef<'__xdevs_inner> ,pub comp_coupled: <Box<Coup<W,P> >as xdevs::traits::Component> ::OutputRef<'__xdevs_inner>
    }
    pub struct ModCoup<const W:usize,const P:usize>{
        pub input:ModCoupInput,pub output:ModCoupOutput,pub t_last:f64,pub t_next:f64,pub components:ModCoupComponents<W,P> ,
    }
    impl <const W:usize,const P:usize>ModCoup<W,P>{
        #[inline]
        pub fn build(comp_atomic:[Atom;
        W],comp_coupled:Box<Coup<W,P> >) -> Self {
            Self {
                input:ModCoupInput::new(),output:ModCoupOutput::new(),t_last:0.0,t_next:f64::INFINITY,components:ModCoupComponents::new(comp_atomic,comp_coupled),
            }
        }

        }
    unsafe impl <const W:usize,const P:usize>xdevs::traits::Component for ModCoup<W,P>{
        type Input = ModCoupInput;
        type Output = ModCoupOutput;
        type InputRef<'__xdevs_ports>  =  &'__xdevs_ports mut ModCoupInput where Self:'__xdevs_ports;
        type OutputRef<'__xdevs_ports>  =  &'__xdevs_ports ModCoupOutput where Self:'__xdevs_ports;
        #[inline]
        fn get_t_last(&self) -> f64 {
            self.t_last
        }
        #[inline]
        fn set_t_last(&mut self,t_last:f64){
            self.t_last = t_last;
        }
        #[inline]
        fn get_t_next(&self) -> f64 {
            self.t_next
        }
        #[inline]
        fn set_t_next(&mut self,t_next:f64){
            self.t_next = t_next;
        }
        #[inline]
        fn get_input(&self) ->  &Self::Input {
            &self.input
        }
        #[inline]
        fn get_input_mut(&mut self) ->  &mut Self::Input {
            &mut self.input
        }
        #[inline]
        fn get_output(&self) ->  &Self::Output {
            &self.output
        }
        #[inline]
        fn get_output_mut(&mut self) ->  &mut Self::Output {
            &mut self.output
        }
        #[inline]
        fn get_ports(&mut self) -> (Self::InputRef<'_> ,Self::OutputRef<'_>){
            (&mut self.input, &self.output)
        }
        #[inline]
        fn get_out_ports(&self) -> Self::OutputRef<'_>{
            &self.output
        }

        }
    unsafe impl <const W:usize,const P:usize>xdevs::traits::PartialCoupled for ModCoup<W,P>{
        type ComponentsInput<'__xdevs_inner>  = ModCoupComponentsInput<'__xdevs_inner,W,P>where Self:'__xdevs_inner;
        type ComponentsOutput<'__xdevs_inner>  = ModCoupComponentsOutput<'__xdevs_inner,W,P>where Self:'__xdevs_inner;
    }



    unsafe impl <const W:usize,const P:usize>xdevs::traits::AbstractSimulator for ModCoup<W,P>{
        #[inline]
        fn start(&mut self,t_start:f64) -> f64 {
            xdevs::traits::Component::set_t_last(self,t_start);
            let mut t_next = f64::INFINITY;
            t_next = f64::min(t_next,xdevs::traits::AbstractSimulator::start(&mut self.components.comp_atomic,t_start));
            t_next = f64::min(t_next,xdevs::traits::AbstractSimulator::start(&mut self.components.comp_coupled,t_start));
            xdevs::traits::Component::set_t_next(self,t_next);
            t_next
        }
        #[inline]
        fn stop(&mut self,t_stop:f64){
            xdevs::traits::AbstractSimulator::stop(&mut self.components.comp_atomic,t_stop);
            xdevs::traits::AbstractSimulator::stop(&mut self.components.comp_coupled,t_stop);
            xdevs::traits::Component::set_t_last(self,t_stop);
            xdevs::traits::Component::set_t_next(self,f64::INFINITY);
        }
        #[inline]
        fn lambda(&mut self,t:f64){
            if t>=xdevs::traits::Component::get_t_next(self){
                xdevs::traits::AbstractSimulator::lambda(&mut self.components.comp_atomic,t);
                xdevs::traits::AbstractSimulator::lambda(&mut self.components.comp_coupled,t);
                let comp_atomic_output = xdevs::traits::Component::get_out_ports(&self.components.comp_atomic);
                let comp_coupled_output = xdevs::traits::Component::get_out_ports(&self.components.comp_coupled);
                let component_outputs:ModCoupComponentsOutput<'_,W,P>  = ModCoupComponentsOutput {
                    comp_atomic:comp_atomic_output,comp_coupled:comp_coupled_output
                };
                <Self as xdevs::Coupled> ::eoc(&component_outputs, &mut self.output);
            }
        }
        #[inline]
        fn delta(&mut self,t:f64) -> f64 {
            {
                let(comp_atomic_input,comp_atomic_output) = xdevs::traits::Component::get_ports(&mut self.components.comp_atomic);
                let(comp_coupled_input,comp_coupled_output) = xdevs::traits::Component::get_ports(&mut self.components.comp_coupled);
                let component_outputs:ModCoupComponentsOutput<'_,const W:usize,const P:usize>  = ModCoupComponentsOutput {
                    comp_atomic:comp_atomic_output,comp_coupled:comp_coupled_output
                };
                let mut component_inputs:ModCoupComponentsInput<'_,const W:usize,const P:usize>  = ModCoupComponentsInput {
                    comp_atomic:comp_atomic_input,comp_coupled:comp_coupled_input
                };
                <Self as xdevs::Coupled> ::eic(&self.input, &mut component_inputs);
                <Self as xdevs::Coupled> ::ic(&component_outputs, &mut component_inputs);
            }let mut t_next = f64::INFINITY;
            t_next = f64::min(t_next,xdevs::traits::AbstractSimulator::delta(&mut self.components.comp_atomic,t));
            t_next = f64::min(t_next,xdevs::traits::AbstractSimulator::delta(&mut self.components.comp_coupled,t));
            xdevs::traits::Component::clear_output(self);
            xdevs::traits::Component::clear_input(self);
            xdevs::traits::Component::set_t_last(self,t);
            xdevs::traits::Component::set_t_next(self,t_next);
            t_next
        }

        }

        impl<const W: usize, const P: usize> xdevs::Coupled for ModCoup<const W: usize, const P: usize> {}



        // impl<const W: usize, const P: usize> Coup<W, P> {
        //     pub fn new_coup_d(period: f64) -> Coup<W, P> {
        //         Coup::CoupD(Atom::new(period))
        //     }

        //     pub fn new_resto_coup(coup: Coup<W, P>) -> Coup<W, P> {
        //         Coup::RestoCoup(Box::new(coup))
        //     }
        // }

        // #[xdevs::coupled2]
        // pub struct ModLI<const W: usize, const P: usize> {
        //     #[components]
        //     atomic: [Atom; W - 1],
        //     coupled: Box<self::ModCoup>,
        // }

        // impl xdevs::Coupled for ModLI {}
    }

    //Model LI
    // pub mod mod_li {
    //     use super::atom;
    //     use super::mod_coup;

    //     #[xdevs::coupled2]
    //     pub struct ModelLI<const W: usize, const P: usize> {
    //         #[components]
    //         comp_atomic: [atom::Atom; W],
    //         comp_coupled: mod_coup::Coup<W, P>,
    //     }

    //     impl<const W: usize, const P: usize> xdevs::Coupled for ModelLI {
    //         fn new() -> Self {
    //             Self {
    //                 comp_atomic: [atom::Atom::new(period: f64); W],
    //                 comp_coupled: mod_coup::Coup::<W, P>::new(),
    //             }
    //         }
    //     }
    */
}

//Implementación manual de AbstracSimulator
unsafe impl<const W: usize, const P: usize> AbstractSimulator for mod_coup::Coup<W, P> {
    fn start(&mut self, t_start: f64) -> f64 {
        match self {
            mod_coup::Coup::CoupD(d) => d.start(t_start),
            mod_coup::Coup::RestoCoup(r) => r.start(t_start),
        }
    }

    fn stop(&mut self, t_stop: f64) {
        match self {
            mod_coup::Coup::CoupD(d) => d.stop(t_stop),
            mod_coup::Coup::RestoCoup(r) => r.stop(t_stop),
        }
    }

    fn lambda(&mut self, t: f64) {
        match self {
            mod_coup::Coup::CoupD(d) => d.lambda(t),
            mod_coup::Coup::RestoCoup(r) => r.lambda(t),
        }
    }

    fn delta(&mut self, t: f64) -> f64 {
        match self {
            mod_coup::Coup::CoupD(d) => d.delta(t),
            mod_coup::Coup::RestoCoup(r) => r.delta(t),
        }
    }
}

//Implementación manual de Component (porque AbstractSimulator requiere component)
unsafe impl<const W: usize, const P: usize> Component for mod_coup::Coup<W, P> {
    type Input = ModCoupInput;
    type Output = ModCoupOutput;
    type InputRef<'a>
        = &'a mut ModCoupInput
    where
        Self: 'a;
    type OutputRef<'a>
        = &'a ModCoupOutput
    where
        Self: 'a;

    fn get_t_last(&self) -> f64 {
        match self {
            mod_coup::Coup::CoupD(d) => d.get_t_last(),
            mod_coup::Coup::RestoCoup(r) => r.get_t_last(),
        }
    }

    fn set_t_last(&mut self, _t_last: f64) {
        match self {
            mod_coup::Coup::CoupD(d) => d.set_t_last(_t_last),
            mod_coup::Coup::RestoCoup(r) => r.set_t_last(_t_last),
        }
    }

    fn get_t_next(&self) -> f64 {
        match self {
            mod_coup::Coup::CoupD(d) => d.get_t_next(),
            mod_coup::Coup::RestoCoup(r) => r.get_t_next(),
        }
    }

    fn set_t_next(&mut self, _t_next: f64) {
        match self {
            mod_coup::Coup::CoupD(d) => d.set_t_next(_t_next),
            mod_coup::Coup::RestoCoup(r) => r.set_t_next(_t_next),
        }
    }

    fn get_input(&self) -> &Self::Input {
        match self {
            mod_coup::Coup::CoupD(d) => d.get_input(),
            mod_coup::Coup::RestoCoup(r) => r.get_input(),
        }
    }

    fn get_input_mut(&mut self) -> &mut Self::Input {
        match self {
            mod_coup::Coup::CoupD(d) => d.get_input_mut(),
            mod_coup::Coup::RestoCoup(r) => r.get_input_mut(),
        }
    }

    fn get_output(&self) -> &Self::Output {
        match self {
            mod_coup::Coup::CoupD(d) => d.get_output(),
            mod_coup::Coup::RestoCoup(r) => r.get_output(),
        }
    }

    fn get_output_mut(&mut self) -> &mut Self::Output {
        match self {
            mod_coup::Coup::CoupD(d) => d.get_output_mut(),
            mod_coup::Coup::RestoCoup(r) => r.get_output_mut(),
        }
    }
}

fn main() {}
