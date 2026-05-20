use std::time::Instant;

use xdevs_devstone::common::*;
use xdevs_devstone::li::*;

fn main() {
    const WIDTH: usize = 20;
    const W: usize = WIDTH - 1;

    let start = Instant::now();

    xdevs_devstone_macros::generate_li!(20, 20);

    //Creación del modelo atómico generador (mete datos en el modelo LI)
    let generator = Generator::new(5);

    //Creación del modelo final (modelo LI + atómico generador que mete datos en el puerto del LI)
    let modelo_final: ModeloFinal<W> = ModeloFinal::build(generator, model_li);
    let duration = start.elapsed();
    println!("Model creation time: {:?}", duration);
    let start = Instant::now();
    let mut simulator = xdevs::simulator::Simulator::new(modelo_final);
    let config = xdevs::simulator::Config::new(
        ::xdevs::Instant::from_secs(0),
        ::xdevs::Instant::from_secs(10),
        1,
        None,
    );
    let duration = start.elapsed();
    println!("Simulator creation time: {:?}", duration);
    let start = Instant::now();
    simulator.simulate_vt(&config);
    let duration = start.elapsed();
    println!("Simulation time: {:?}", duration);
}

//Funciones que van a ejecutarse cuando haga tests:
#[cfg(test)]
mod test {
    use super::*;
    fn expected_n_atomic(width: usize, profundidad: usize) -> usize {
        //puede comprobarse antes de simulate_vt
        (width - 1) * (profundidad - 1) + 1
    }

    fn expected_n_events(width: usize, profundidad: usize) -> usize {
        (width - 1) * (profundidad - 1) + 1
    }

    #[test]
    fn test_li() {
        const WIDTH: usize = 5;
        const DEPTH: usize = 1;
        const W: usize = WIDTH - 1;

        //Creación del modelo acoplado total LI
        xdevs_devstone_macros::generate_li!(5, 1);
        //Creación del modelo atómico generador (mete datos en el modelo LI)
        let generator = Generator::new(5);
        //Creación del modelo final (modelo LI + atómico generador que mete datos en el puerto del LI)
        let modelo_final: ModeloFinal<W> = ModeloFinal::build(generator, model_li);
        let mut simulator = xdevs::simulator::Simulator::new(modelo_final);
        let config = xdevs::simulator::Config::new(
            ::xdevs::Instant::from_secs(0),
            ::xdevs::Instant::from_secs(10),
            1,
            None,
        );
        simulator.simulate_vt(&config);
        let modelo_final = simulator.get_model();

        assert_eq!(
            expected_n_atomic(WIDTH, DEPTH),
            modelo_final.get_n_atomics()
        );
        assert_eq!(expected_n_events(WIDTH, DEPTH), modelo_final.get_n_events());
        assert_eq!(
            modelo_final.get_n_internals(),
            modelo_final.get_n_externals()
        );
    }
}
