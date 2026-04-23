use crate as xdevs;
#[crate::atomic]
#[derive(Debug, Clone, PartialEq, Copy, Eq)]
pub struct Atomic1{
    #[input]
    in_port: xdevs::port::Port<bool, 1>,
    #[output]
    out_port: xdevs::port::Port<bool, 1>,
    #[state]
    sigma: f64,
}

impl xdevs::Atomic for Atomic1 {
    fn delta_int(state: &mut Self::State) {
        state.sigma = 10.0;
    }

    fn lambda(state: &Self::State, output: &mut Self::Output) {
        output.out_port.add_value(1);
    }

    fn ta(state: &Self::State) -> f64 {
        state.sigma
    }

    fn delta_ext(state: &mut Self::State, e: f64, x: &Self::Input) {
        state.sigma -= e;
    }
}


#[xdevs::coupled(
    couplings = {
        Atomic1.out_port -> out,
    }
)]

pub struct DEVStoneEnd {
    #[components]
    atomic: Atomic1,
}
 




//#[coupled(in_port -> atomics[i].in_port)]
pub struct DEVStoneLI<'a, const W: usize> {
    coupled: LI<'a, W>,
    atomics: [Atomic1; N]
}
 
enum LI<'a, const W: usize> {
    End(DEVStoneEnd),
    Normal(&'a mut DEVStoneLI<'a, W>)
}
 
fn lambda(&self) {
    match self {
        End(m) => m.lambda,
        Normal(m) => m.lambda,
    }
}
 
fn main() {
    let mut depth1 = LI::End(DevstoneEnd);
    let mut depth2 = LI::Normal(DEVStoneLi<W>(&mut depth1));
    let mut depth3 = DEVStoneLi<W>(&mut depth2);
    let mut depth4 = DEVStoneLi<W>(&mut depth3);
}