use std::{iter::Sum, usize, ops::{Mul, Add, Sub, Div}};

fn main() {
    println!("Hello, world!");
}

#[derive(Default, Debug, PartialEq)]
struct Signal<T,U>
where T: Default + Clone,
      U: Default + Clone
{
    /* Samples of a digital signal */
    pub v: Vec<Sample<T,U>>
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
struct Sample<T,U>
where T: Default + Clone,
      U: Default + Clone
{
    /* time compenent of the sample */
    pub t: T,
    
    /* measured value */
    pub x: U
}

impl<T,U> Signal<T,U>
where T: Default + Clone,
      U: Default + Clone
{
    /* create a new signal base of the time and value component */
    pub fn new(time_signal: Vec<T>, value_signal: Vec<U>) -> Signal<T,U>{
        assert_eq!(time_signal.len(), value_signal.len());
        Signal { 
            v: value_signal.iter().zip(time_signal.iter())
                .map(|(x_val,t_val)| Sample{t:t_val.clone(), 
                                                         x:x_val.clone()})
                .collect()
                }
    }    
    
    /* Perform a time discrete signal fold, the signals are resampled using the time sample_time:*/
    pub fn fold_signal<'a>(&'a self, signalb: &'a Self, sample_time: &'a T) -> Option<Self>
    where T:      PartialOrd + 
                       Add<&'a T, Output = T> + Sub<&'a T, Output = T> + Div<Output=T> + Mul<&'a T, Output = T> +
                       TryInto<U> + TryInto<usize> + TryFrom<usize>  + From<u8> + 'a,
              U:      Mul<Output = U> + From<u8> + Sum,
              &'a T: Sub<&'a T, Output = T> + Mul<&'a T, Output = T> + 'a,
              &'a U: Add<U,Output = U> + Sub<&'a U, Output = U> +'a
    {
        
        if *sample_time <= 0_u8.into(){
            return None;
        }

        /* Resample the input signals and add zeros to ease zipping */
        let t_min_siga = &self.v.first()?.t;
        let t_min_sigb = &signalb.v.first()?.t;
        let sig_a;
        let sig_b;
        let len_diff;
        if t_min_siga > t_min_sigb
        {
            len_diff = ((t_min_siga - t_min_sigb)/(sample_time.clone())).try_into().ok()?;
            sig_a = [vec![0_u8.into();len_diff],self.resample(sample_time)?].concat();
            sig_b = [vec![0_u8.into();sig_a.len() - 1], signalb.resample(sample_time)?].concat();

        }else{
            len_diff = ((t_min_sigb - t_min_siga)/(sample_time.clone())).try_into().ok()?;
            sig_a = self.resample(sample_time)?;
            sig_b = [vec![0_u8.into();len_diff + sig_a.len() - 1], signalb.resample(sample_time)?].concat();
        }
        
        /* Generate an vector for the results */
        let len_result = sig_b.len();
        let len_a = sig_a.len();
        let t_min_result = if t_min_siga < t_min_sigb {t_min_siga} else {t_min_sigb};
        let mut result = vec![0_u8.into(); len_result];

        /* Perfor the signal fold */
        for (index, sample) in result.iter_mut().enumerate(){
            *sample = sig_a.iter()
                                    .zip(sig_b.iter().skip(index).take(len_a).rev())
                                    .map(|(a,b)| a.clone()*b.clone())
                                    .sum();

        }

		/* Generate time signal and return result */
        Some(Signal::new((0..len_result).into_iter()
                                                     .map(|i| 
                                                             match T::try_from(i){ Ok(i_ok) => (i_ok*sample_time) + t_min_result,
                                                                                                       _ => 0_u8.into()})
                                                     .collect(),
            result))
    }
        
    /* Resample a signal with a given sampletime */
    fn resample<'a>(&'a self, sampletime: &'a T) -> Option<Vec<U>>
    where T:      PartialOrd + Add<&'a T, Output = T> + Sub<&'a T, Output = T> + Div<Output=T> + TryInto<U> +'a,
              U:      Mul<Output = U> + From<u8>,
              &'a T: Sub<&'a T, Output = T> + 'a,
              &'a U: Add<U,Output = U> + Sub<&'a U, Output = U> + 'a
    {
        let first_sample = match self.v.first() {Some(s) => s, _ => return None};

        let mut result = Vec::new();
        result.push(first_sample.x.clone());
        let mut new_t = first_sample.t.clone();

        for i in 1..self.v.len(){
            while new_t < self.v[i].t{
                new_t = new_t + sampletime;
                let new_x = &self.v[i-1].x +
                                   (&self.v[i].x-&self.v[i-1].x) *
                                    (((new_t.clone()-&self.v[i-1].t))/(&self.v[i].t-&self.v[i-1].t))
                                    .try_into()
                                    .ok()?;
                result.push(new_x);
            }
        }
        Some(result)
    }

}




#[cfg(test)]
mod tests {
    use crate::Signal;

    #[test]
    fn impuls_response() {
        //Preparation
        let signal = Signal::new( vec![0u32,1,2,3], vec![1.0f64,2.0,3.0,4.0]);
        let signal_b = Signal::new(vec![0u32,1,2,3], vec![1.0f64,0.0,0.0,0.0]);
        let sample_time = 1;

        //Calculation
        let signal_c = signal.fold_signal(&signal_b, &sample_time).unwrap();
        
        //Ceck
        assert_eq!(signal_c, Signal::new( vec![0u32,1,2,3,4,5,6], vec![1.0f64,2.0,3.0,4.0,0.0,0.0,0.0]));
    }

    #[test]
    fn impuls_response_timeshift() {
        //Preparation
        let signal = Signal::new( vec![0u32,1,2,3], vec![1.0f64,2.0,3.0,4.0]);
        let signal_b = Signal::new(vec![1u32,2,3,4], vec![1.0f64,0.0,0.0,0.0]);
        let sample_time = 1;

        //Calculation
        let signal_c = signal.fold_signal(&signal_b, &sample_time).unwrap();
        
        //Ceck
        assert_eq!(signal_c, Signal::new( vec![0u32,1,2,3,4,5,6,7], vec![0.0f64,1.0,2.0,3.0,4.0,0.0,0.0,0.0]));
    }

    #[test]
    fn impuls_response_scaled() {
        //Preparation
        let signal = Signal::new( vec![0u32,1,2,3], vec![1.0f64,2.0,3.0,4.0]);
        let signal_b = Signal::new(vec![1u32,2,3,4], vec![2.0f64,0.0,0.0,0.0]);
        let sample_time = 1;

        //Calculation
        let signal_c = signal.fold_signal(&signal_b, &sample_time).unwrap();
        
        //Ceck
        assert_eq!(signal_c, Signal::new( vec![0u32,1,2,3,4,5,6,7], vec![0.0f64,2.0,4.0,6.0,8.0,0.0,0.0,0.0]));
    }
}