use std::time::Instant;

use xdevs_devstone::common::*;
use xdevs_devstone::ho::*;

fn main() {
    const WIDTH: usize = 20;
    const W: usize = WIDTH - 1;

    let start = Instant::now();

    xdevs_devstone_macros::generate_ho!(20, 20);

    //Creación del modelo atómico generador (mete datos en el modelo HO)
    let generator = Generator::new(5);

    //Creación del modelo final (modelo HO + atómico generador que mete datos en el puerto del HO)
    let modelo_final: ModeloFinal<W> = ModeloFinal::build(generator, model_ho);
    let duration = start.elapsed();
    println!("Model creation time: {:?}", duration);
    let start = Instant::now();
    let mut simulator = xdevs::simulator::Simulator::new(modelo_final);
    let config = xdevs::simulator::Config::new(0.0, 10.0, 1.0, None);
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

    //CAMBIAR ESTA ECUACIÓN
    fn expected_n_events(width: usize, profundidad: usize) -> usize {
        1 + (profundidad - 1) * ((width - 1) * width) / 2
    }

    #[test]
    fn test_ho() {
        const WIDTH: usize = 1;
        const DEPTH: usize = 1;
        const W: usize = WIDTH - 1;

        xdevs_devstone_macros::generate_ho!(1, 1);

        //Creación del modelo atómico generador (mete datos en el modelo HO)
        let generator = Generator::new(5);

        //Creación del modelo final (modelo HO + atómico generador que mete datos en el puerto del HO)
        let modelo_final: ModeloFinal<W> = ModeloFinal::build(generator, model_ho);
        let mut simulator = xdevs::simulator::Simulator::new(modelo_final);
        let config = xdevs::simulator::Config::new(0.0, 10.0, 1.0, None);
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
