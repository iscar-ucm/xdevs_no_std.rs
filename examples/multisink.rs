mod multisink {
    use xdevs::port::Port;
    
    #[xdevs::atomic]
    pub struct MultiSink {
        #[input]
        pub in_jobs: [Port<usize, 1>; 4],

        #[state]
        sigma: f64,
        total_received: usize,
    }

    impl xdevs::Atomic for MultiSink {
        fn delta_int(state: &mut Self::State) {
            state.sigma = f64::INFINITY;
        }

        fn lambda(_state: &Self::State, _output: &mut Self::Output) {
        }

        fn ta(state: &Self::State) -> f64 {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, e: f64, x: &Self::Input) {
            state.sigma -= e;

            for (i, port) in x.in_jobs.iter().enumerate() {
                for &job in port.get_values() {
                    state.total_received += 1;
                    println!("[MultiSink] port {} received job {}", i, job);
                }
            }
        }
    }

    impl MultiSink {
        pub fn new2() -> Self {
            Self::new(f64::INFINITY, 0)
        }
    }
}

mod multisource {
    use xdevs::port::Port;

    #[xdevs::atomic]
    pub struct MultiSource {
        #[output]
        pub out_jobs: [Port<usize, 1>; 4],

        #[state]
        sigma: f64,
        counter: usize,
    }

    impl xdevs::Atomic for MultiSource {
        fn delta_int(state: &mut Self::State) {
            state.counter += 1;
            state.sigma = 1.0;
        }

        fn lambda(state: &Self::State, output: &mut Self::Output) {
            for (i, port) in output.out_jobs.iter_mut().enumerate() {
                println!("[MultiSource] sending job {} to port {}", state.counter, i);
                port.add_value(state.counter).unwrap();
            }
        }

        fn ta(state: &Self::State) -> f64 {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, e: f64, _x: &Self::Input) {
            state.sigma -= e;
        }
    }

    impl MultiSource {
        pub fn new2() -> Self {
            Self::new(1.0, 0)
        }
    }
}

#[xdevs::coupled(
    couplings = {
        source.out_jobs[0] -> sink.in_jobs[0],
        source.out_jobs[1] -> sink.in_jobs[1],
    }
)]
struct TestModel {
    #[components]
    source: multisource::MultiSource,
    sink: multisink::MultiSink,
}

fn main() {
    let source = multisource::MultiSource::new2();
    let sink = multisink::MultiSink::new2();
    
    let test_model = TestModel::new(source, sink);

    let mut simulator = xdevs::simulator::Simulator::new(test_model);
    let config = xdevs::simulator::Config::new(0.0, 5.0, 1.0, None);
    simulator.simulate_rt(&config, xdevs::simulator::std::sleep(&config), |_| {});
}
