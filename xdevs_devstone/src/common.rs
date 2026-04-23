use xdevs::traits::{AbstractSimulator, Component};

pub static mut N_EIC: usize = 0;
pub static mut N_EOC: usize = 0;
pub static mut N_IC: usize = 0;

pub fn get_n_eic() -> usize {
    unsafe { N_EIC }
}

pub fn get_n_eoc() -> usize {
    unsafe { N_EOC }
}

pub fn get_n_ic() -> usize {
    unsafe { N_IC }
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

//Atomic model
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

    fn delta_ext(state: &mut Self::State, elapsed: f64, input: &Self::Input) {
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
        let n_internals_atom = self.state.n_internals;
        print!(
            "Número de eventos internos del atómico: {}",
            n_internals_atom
        );
        n_internals_atom
    }

    pub fn get_n_externals(&self) -> usize {
        let n_externals_atom = self.state.n_externals;
        print!(
            "Número de eventos externos del atómico: {}",
            n_externals_atom
        );
        n_externals_atom
    }

    pub fn get_n_events(&self) -> usize {
        let n_events_atom = self.state.n_events;
        print!("Número de eventos del atómico: {}", n_events_atom);
        n_events_atom
    }
}
//Fin atomic model
