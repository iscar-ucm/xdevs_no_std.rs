mod bloom_model{
    
    #[derive(Clone, Debug, Default)]
    pub struct ExitData{
        pub id: String,
        pub source: String,
        pub timestamp: f64,
        pub lat: f64,
        pub lon: f64,
        pub breath: f64,
        pub photo: f64,
        pub size: f64,
        pub is_bloom: bool,
    }

    struct  InputData{
        dox: f64,
        nox: f64,
        sun: f64,
        wfu: f64,
        wfv: f64,
    }

    impl  InputData {
        fn all_values_ok(&self) -> bool {
            // La idea es identificar si hemos recibido todos los valores
            let threshold = f64::NEG_INFINITY;
            if self.dox > threshold && 
                self.nox > threshold && 
                self.sun > threshold && 
                self.wfu > threshold && 
                self.wfv > threshold {
                    //println!("[ALL VALUES OK] dox = {}, nox = {}, sun = {}, wfu = {}, wfv = {}", 
                    //    self.dox, self.nox, self.sun, self.wfu, self.wfv);
                return true
            }

            false
        }
    }

    pub struct BMState{
        sigma: f64,

        k1: f64,
        k2: f64,
        k3: f64,
        k2_2d_dis_bloom: f64,
        name: String, 
        bloom_lat: f64, 
        bloom_lat_ini: f64,
        bloom_lon: f64, 
        bloom_lon_ini: f64,
        bloom_size: f64,
        is_bloom: bool,
        breath: f64,
        photo: f64,
        clock: f64,
        
        
        in_data: InputData,

    }

    impl  BMState {
        pub fn new(name: String, lat_ini: f64, lon_ini: f64, size_ini: f64) -> Self {
            Self {
                sigma: 0.0,
                
                k1: 5.0,
                k2: 0.05,
                k3: 1.0/6.0,
                k2_2d_dis_bloom: 1.0/60.0,
                name,

                bloom_lat_ini: lat_ini,
                bloom_lon_ini: lon_ini,

                bloom_lat: lat_ini,
                bloom_lon: lon_ini,

                bloom_size: size_ini,
                is_bloom: false,
                breath: 0.0,
                photo: 0.0,
                clock: 0.0, 
                
                in_data: InputData{
                    dox: f64::NEG_INFINITY,
                    nox: f64::NEG_INFINITY,
                    sun: f64::NEG_INFINITY,
                    wfu: f64::NEG_INFINITY,
                    wfv: f64::NEG_INFINITY,
                },

            }
        }
    }

    
    xdevs::component!(
        ident = BloomModel,
        input = {
            iport_cmd <String>,
            iport_dox <f64>,
            iport_nox <f64>,
            iport_sun <f64>,
            iport_wfu <f64>,
            iport_wfv <f64>,
        },
        output = {
            oprot_out<ExitData>,
        },
        state = BMState,
    );

    

    impl xdevs::Atomic for BloomModel {

        fn start(state: &mut Self::State) {
            state.sigma = f64::INFINITY; // Passivate
            
            
        }

        fn delta_int(state: &mut Self::State) {
            state.clock += state.sigma;

            /*          
            if state.clock % 86400.0 == 0. || state.clock < 1.0{
                println!("<< RESET BLOOM SIZE >>");
                state.bloom_size = 0.0;
                state.bloom_lon = state.bloom_lon_ini;
                state.bloom_lat = state.bloom_lat_ini;
                state.is_bloom = false;
            }
            */

            
            state.in_data.dox = f64::NEG_INFINITY;
            state.in_data.nox = f64::NEG_INFINITY;
            state.in_data.sun = f64::NEG_INFINITY;
            state.in_data.wfu = f64::NEG_INFINITY ;
            state.in_data.wfv = f64::NEG_INFINITY;
            

            state.sigma = f64::INFINITY; // Passivate
        }

        fn delta_ext(state: &mut Self::State, e: f64, x: &Self::Input) {
            state.sigma -= e;
            

            // Tratar puerto cmd por ahora omitido
            if !x.iport_cmd.is_empty(){
                if let Some(data) = x.iport_cmd.get_values().last(){
                    println!("## CMD: {} ##",data);
                    if data == "START"{
                        state.clock = 0.0; // TODO cmd.date
                        // initialize (start)
                    }else if data == "STOP"{
                        // stop
                        state.sigma = f64::INFINITY; // Passivate
                    }else if data == "RESET"{
                        // reset
                        println!("<< RESET BLOOM SIZE >>");
                        state.bloom_size = 0.0;
                        state.bloom_lon = state.bloom_lon_ini;
                        state.bloom_lat = state.bloom_lat_ini;
                        state.is_bloom = false;                    
                    }
                }
            }
            state.clock += e;
            

            // Tratar el resto de puertos
            if !x.iport_dox.is_empty(){
                if let Some(data)= x.iport_dox.get_values().last(){
                    state.in_data.dox = *data;
                }
            }
            if !x.iport_nox.is_empty(){
                if let Some(data)= x.iport_nox.get_values().last(){
                    state.in_data.nox = *data;
                }
            }
            if !x.iport_sun.is_empty(){
                if let Some(data)= x.iport_sun.get_values().last(){
                    state.in_data.sun = *data;
                }
            }
            if !x.iport_wfu.is_empty(){
                if let Some(data)= x.iport_wfu.get_values().last(){
                    state.in_data.wfu = *data;
                }
            }
            if !x.iport_wfv.is_empty(){
                if let Some(data)= x.iport_wfv.get_values().last(){
                    state.in_data.wfv = *data;
                }
            }
            // Si recibimos valores en todos los puertos actualizamos el estado
            if state.in_data.all_values_ok(){

                // Update state
                if state.in_data.dox > 20.0 {
                    state.is_bloom = true;
                } else if state.in_data.dox < 15.0{
                    state.is_bloom = false;
                } 
                
                if state.is_bloom {
                    state.bloom_lat = state.bloom_lat + state.k2_2d_dis_bloom * state.in_data.wfv;
                    state.bloom_lon = state.bloom_lon + state.k2_2d_dis_bloom * state.in_data.wfu;
                } else {
                    state.bloom_lat = state.bloom_lat_ini;
                    state.bloom_lon = state.bloom_lon_ini;
                }

                state.breath = state.in_data.dox * state.in_data.nox;
                state.photo = state.in_data.sun * state.in_data.nox;

                state.bloom_size = state.bloom_size + state.k1 * state.photo + state.k2 * state.breath - state.k3 * state.bloom_size;

                // Size control
                if state.bloom_size > 10.0{
                    state.bloom_size = 10.0;
                }

                println!("BLOOM: [lat = {}, lon= {}, size= {}]", state.bloom_lat, state.bloom_lon, state.bloom_size);

                state.sigma = 0.0; // Activate, SI se han recibido 5 inputs
            }

           
        }

        fn lambda(state: &Self::State, output: &mut Self::Output) {
            // Tratar datos de salida
            let data = ExitData{
                id: "BLOOM".to_string(),
                source: state.name.clone(),
                timestamp: state.clock,
                lat: state.bloom_lat,
                lon: state.bloom_lon,
                breath: state.breath,
                photo: state.photo,
                size: state.bloom_size,
                is_bloom: state.is_bloom,
            };
            output.oprot_out.add_value(data).unwrap();
        }

        fn ta(state: &Self::State) -> f64 {
            state.sigma
        }

        
    }

}

mod generator_model{
    use crate::bloom_model::{self, ExitData};


    pub struct GMState{
        sigma: f64,
        sigma_pre: f64,

        writer: csv::Writer<std::fs::File>,
        reader: csv::Reader<std::fs::File>,
        // Campos del CSV
        //timestamp,lat,lon,ALG,BTH,NOX,DOX,sun,temperature,U,V,wind_x,wind_y
        timestamp: f64,
        lat: f64,
        lon: f64,
        alg: f64,
        bth: f64,
        nox: f64,
        dox: f64,
        sun: f64,
        temperature: f64,
        u: f64,
        v: f64,
        wind_x: f64,
        wind_y: f64, 

        last_row: bool,       
    }

    impl GMState{
        pub fn new(ifp:&str, ofp:String) -> Self {
            Self {
                sigma: 0.0,
                sigma_pre: 0.0,

                writer: csv::Writer::from_path(ofp).expect("[W] No se pudo abrir el archivo al crearse."),
                reader: csv::Reader::from_path(ifp).expect("[R] No se pudo abrir el archivo al crearse."),
                
                timestamp: 0.0,
                lat: 0.0,
                lon: 0.0,
                alg: 0.0,
                bth: 0.0,
                nox: 0.0,
                dox: 0.0,
                sun: 0.0,
                temperature: 0.0,
                u: 0.0,
                v: 0.0,
                wind_x: 0.0,
                wind_y: 0.0,
                
                last_row: false,
            }
        }
        pub fn reset_values (&mut self){
            // Resetear todos los campos a 0
            self.timestamp =  0.0;
            self.lat = 0.0;
            self.lon = 0.0;
            self.alg = 0.0;
            self.bth = 0.0;
            self.nox = 0.0;
            self.dox = 0.0;
            self.sun = 0.0;
            self.temperature = 0.0;
            self.u = 0.0;
            self.v = 0.0;
            self.wind_x = 0.0;
            self.wind_y = 0.0;
        } 
        
        pub fn read_data(&mut self, sigma_pre: f64){
            // TODO ver que hacer con sigma cuando no sea tan periodico

            match self.reader.records().next(){
                Some(Ok(record)) => {
                    self.sigma = (record[0].parse::<f64>().unwrap() - sigma_pre) * 60.0; // Pasar a segundos
                    self.sigma_pre = record[0].parse::<f64>().unwrap();
                    self.timestamp = record[0].parse::<f64>().unwrap();
                    self.lat = record[1].parse::<f64>().unwrap();
                    self.lon = record[2].parse::<f64>().unwrap();
                    self.alg = record[3].parse::<f64>().unwrap();
                    self.bth = record[4].parse::<f64>().unwrap();
                    self.nox = record[5].parse::<f64>().unwrap();
                    self.dox = record[6].parse::<f64>().unwrap();
                    self.sun = record[7].parse::<f64>().unwrap();
                    self.temperature = record[8].parse::<f64>().unwrap();
                    self.u = record[9].parse::<f64>().unwrap();
                    self.v = record[10].parse::<f64>().unwrap();
                    self.wind_x = record[11].parse::<f64>().unwrap();
                    self.wind_y = record[12].parse::<f64>().unwrap();
                    //println!("READING DATA: [timestamp = {}, lat = {}, lon = {}, alg = {}, bth = {}, nox = {}, dox = {}, sun = {}, temperature = {}, u = {}, v = {}, wind_x = {}, wind_y = {}]", self.timestamp, self.lat, self.lon, self.alg, self.bth, self.nox, self.dox, self.sun, self.temperature, self.u, self.v, self.wind_x, self.wind_y);
                    
                },
                Some(Err(err)) => panic!("[R][0] No se pudo leer el registro: {}", err),
                None => {
                    self.last_row = true;
                    println!("FIN DE FICHERO DE ENTRADA!")
                }, 
            }
        }
        pub fn write_data(&mut self, exit_data: &ExitData){
            // Guardar datos en el csv
            self.writer.write_record(&[
                exit_data.id.to_string(),
                exit_data.source.to_string(),
                exit_data.timestamp.to_string(),
                exit_data.lat.to_string(),
                exit_data.lon.to_string(),
                exit_data.breath.to_string(),
                exit_data.photo.to_string(),
                exit_data.size.to_string(),
                exit_data.is_bloom.to_string(),
                ]).expect("[W] No se pudo escribir el registro");
            
        }
    }

    xdevs::component!(
        ident = GeneratorModel,
        input = {
            iport_data_collected<bloom_model::ExitData>,
        },
        output = {
            oport_cmd <String>, 
            oport_dox <f64>,
            oport_nox <f64>,
            oport_sun <f64>,
            oport_wfu <f64>,
            oport_wfv <f64>,
        },
        state = GMState,
    );

    impl xdevs::Atomic for GeneratorModel {
         
        fn start(state: &mut Self::State) {
            state.writer.write_record(&["id","source","timestamp","lat","lon","breath","photo","size","is_bloom"]).expect("[W] No se pudo escribir el registro inicial");
            state.sigma_pre = 0.0;
            state.read_data(state.sigma_pre);
            
            
        }
        

        fn delta_ext(state: &mut Self::State, e: f64, x: &Self::Input) {
            // Recopilar los datos de entrada y guardarlos en un archivo
            state.sigma -= e;
            if !x.iport_data_collected.is_empty(){
                if let Some(data) = x.iport_data_collected.get_values().last(){
                    
                    state.write_data(data);
                    if state.last_row{
                        panic!("Fin del fichero y fin de ejecución!")
                    }
                    
                }
            }       
        }
        fn delta_int(state: &mut Self::State) {
            // Leer los datos de entrada y gestionarlos
            state.read_data(state.sigma_pre);             
               
            
        }
        fn lambda(state: &Self::State, output: &mut Self::Output) {
            // Mandar datos
            output.oport_dox.add_value(state.dox).unwrap();
            output.oport_nox.add_value(state.nox).unwrap();
            output.oport_sun.add_value(state.sun).unwrap();
            output.oport_wfu.add_value(state.u).unwrap();
            output.oport_wfv.add_value(state.v).unwrap();
            if state.timestamp % 1440.0 == 0.0{
                println!("Timestamp is = {}", state.timestamp);
                println!("Module is = {}", state.timestamp % 1440.0);
                output.oport_cmd.add_value("RESET".to_string()).unwrap();
            }
            //output.oport_cmd.add_value("RESET".to_string()).unwrap();
        }
        fn ta(state: &Self::State) -> f64 {
            state.sigma
        }
    }
}


xdevs::component!(
    ident = SYS,
    components = {
        bloom_model: bloom_model::BloomModel,
        generator_model: generator_model::GeneratorModel,
    },
    couplings = {
        bloom_model.oprot_out -> generator_model.iport_data_collected,
        generator_model.oport_cmd -> bloom_model.iport_cmd, // TODO ver que hacer con esto ¿quitar?
        generator_model.oport_dox -> bloom_model.iport_dox,
        generator_model.oport_nox -> bloom_model.iport_nox,
        generator_model.oport_sun -> bloom_model.iport_sun,
        generator_model.oport_wfu -> bloom_model.iport_wfu,
        generator_model.oport_wfv -> bloom_model.iport_wfv,
    }
);

fn main(){
    let bm = bloom_model::BloomModel::new(bloom_model::BMState::new("BLOOM".to_string(), 47.5064888000488, -122.216850280762, 0.0));
    

    

    //let ifp = r"C:\Users\Usuario\Desktop\00 UNI\000 UCM\xdevs_no_std.rs\examples\input.csv";
    let ifp = "examples/input.csv";
    //let ifp = r"C:\Users\Usuario\Desktop\00 UNI\000 UCM\xdevs_no_std.rs\examples\input_different_time_inputs.csv";
    let ofp = "output.csv".to_string();
    let gen = generator_model::GeneratorModel::new(generator_model::GMState::new(ifp, ofp));
    
    let sys = SYS::new(bm, gen);

    let mut simulator = xdevs::simulator::Simulator::new(sys);
    simulator.simulate_vt(0., 60.*13000.);

}  
