use core::f64;
use std::time::Instant;

use xdevs_devstone::common::*;
use xdevs_devstone::li::*;

fn main() {
    const WIDTH: usize = 5; //a lo mejor me toca sacar WIDTH y DEPTH del main y hacerlas globales para que los tests puedan usarlas
    const DEPTH: usize = 71;
    const W: usize = WIDTH - 1;

    let start = Instant::now();

    //Creación de modelo LI
    // let atom = CoupAtom::new(); //Modelo atómico que va dentro del acoplado (es el acoplado más interno del modelo LI, es CoupD)
    // let coup_atom_d: Coup<W> = Coup::CoupD(atom);
    // let modelo_li: ModCoupLI<W> =
    //     ModCoupLI::build(core::array::from_fn(|_| Atom::new()), Box::new(coup_atom_d));
    // let modelo_li_2: ModCoupLI<W> = ModCoupLI::build(
    //     core::array::from_fn(|_| Atom::new()),
    //     Box::new(Coup::RestoCoup(modelo_li)),
    // );
    // let modelo_li_3: ModCoupLI<W> = ModCoupLI::build(
    //     core::array::from_fn(|_| Atom::new()),
    //     Box::new(Coup::RestoCoup(modelo_li_2)),
    // );
    // let modelo_li_4: ModCoupLI<W> = ModCoupLI::build(
    //     core::array::from_fn(|_| Atom::new()),
    //     Box::new(Coup::RestoCoup(modelo_li_3)),
    // );
    xdevs_devstone_macros::generate_li!(2, 5);

    //Creación del modelo atómico generador (mete datos en el modelo LI)
    let generator = Generator::new(5);

    //Creación del modelo final (modelo LI + atómico generador que mete datos en el puerto del LI)
    let modelo_final = ModeloFinal::build(generator, model_li);
    let duration = start.elapsed();
    println!("Model creation time: {:?}", duration);
    let start = Instant::now();
    let mut simulator = xdevs::simulator::Simulator::new(modelo_final);
    let config = xdevs::simulator::Config::new(0.0, 10.0, 1.0, None);
    let duration = start.elapsed();
    println!("Simulator creation time: {:?}", duration);
    //simulator.simulate_rt(&config, xdevs::simulator::std::sleep(&config), |_| {});
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
        let n_atomic_expected = (width - 1) * (profundidad - 1) + 1;
        // println!("Número de atómicos esperados: {}", n_atomic_expected);
        n_atomic_expected
    }
    fn expected_eic(width: usize, profundidad: usize) -> usize {
        let n_eic_expected = width * (profundidad - 1) + 1;
        // println!("Número de eics esperados: {}", n_eic_expected);
        n_eic_expected
    }
    fn expected_eoc(profundidad: usize) -> usize {
        let n_eoc_expected = profundidad;
        // println!("Número de eocs esperados: {}", n_eoc_expected);
        n_eoc_expected
    }
    fn expected_ic() -> usize {
        let n_ic_expected = 0;
        // println!("Número de ics esperados: {}", n_ic_expected);
        n_ic_expected
    }
    fn expected_n_events(width: usize, profundidad: usize) -> usize {
        let n_events_expected = (width - 1) * (profundidad - 1) + 1;
        // println!("Número de eventos esperados: {}", n_events_expected);
        n_events_expected
    }

    #[test]
    fn test_li() {
        const WIDTH: usize = 5; //a lo mejor me toca sacar WIDTH y DEPTH del main y hacerlas globales para que los tests puedan usarlas
        const DEPTH: usize = 71;
        const W: usize = WIDTH - 1;
        // let atom = CoupAtom::new(); //Modelo atómico que va dentro del acoplado (es el acoplado más interno del modelo LI, es CoupD)
        // let coup_atom_d: Coup<W> = Coup::CoupD(atom);
        // let modelo_li: ModCoupLI<W> =
        //     ModCoupLI::build(core::array::from_fn(|_| Atom::new()), Box::new(coup_atom_d));
        // let modelo_li_2: ModCoupLI<W> = ModCoupLI::build(
        //     core::array::from_fn(|_| Atom::new()),
        //     Box::new(Coup::RestoCoup(modelo_li)),
        // );
        // let modelo_li_3: ModCoupLI<W> = ModCoupLI::build(
        //     core::array::from_fn(|_| Atom::new()),
        //     Box::new(Coup::RestoCoup(modelo_li_2)),
        // );
        // let modelo_li_4: ModCoupLI<W> = ModCoupLI::build(
        //     core::array::from_fn(|_| Atom::new()),
        //     Box::new(Coup::RestoCoup(modelo_li_3)),
        // );

        //Creación del modelo acoplado total LI
        xdevs_devstone_macros::generate_li!(85, 83);
        //Creación del modelo atómico generador (mete datos en el modelo LI)
        let generator = Generator::new(5);
        // //Creación del modelo final (modelo LI + atómico generador que mete datos en el puerto del LI)
        // let modelo_final = ModeloFinal::build(generator, coup_atom_d);
        let modelo_final: ModeloFinal<W> = ModeloFinal::build(generator, model_li);
        let mut simulator = xdevs::simulator::Simulator::new(modelo_final);
        let config = xdevs::simulator::Config::new(0.0, 10.0, 1.0, None);
        simulator.simulate_vt(&config);
        let modelo_final = simulator.get_model();

        assert_eq!(
            expected_n_atomic(WIDTH, DEPTH),
            modelo_final.get_n_atomics()
        );
        assert_eq!(expected_eic(WIDTH, DEPTH), modelo_final.get_n_eic());
        assert_eq!(expected_eoc(DEPTH), modelo_final.get_n_eoc());
        assert_eq!(expected_ic(), modelo_final.get_n_ic());
        assert_eq!(expected_n_events(WIDTH, DEPTH), modelo_final.get_n_events());
        assert_eq!(
            modelo_final.get_n_internals(),
            modelo_final.get_n_externals()
        );
    }
}