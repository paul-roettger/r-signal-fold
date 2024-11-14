use std::{iter::Sum, usize, ops::{Mul, Add, Sub, Div}, f64, convert::FloatToInt};
#[macro_use] extern crate custom_derive;
#[macro_use] extern crate newtype_derive;


fn main() {
    println!("Hello, world!");
}


custom_derive! {
    #[derive(NewtypeFrom,
        NewtypeAdd, NewtypeDiv, NewtypeMul, NewtypeSub,
        NewtypeDeref, NewtypeDerefMut, NewtypeUpperExp
        )]
    pub struct F64T(f64);
}

#[derive(Default)]
struct Signal<T,U>
where T: Default + Clone,
      U: Default + Clone
{
    pub v: Vec<Sample<T,U>>
}

#[derive(Default, Clone, Copy)]
struct Sample<T,U>
where T: Default + Clone,
      U: Default + Clone
{
    pub t: T,
    pub x: U
}

trait FoldableSignal{
    type TypeTime;
    fn fold_signal<'a>(&'a self, signalb: &'a Self, sample_time: &'a Self::TypeTime) -> Option<Self> where Self: Sized;
}


impl<'a,T,U> FoldableSignal for Signal<T,U> 
where T: Default + Clone + PartialOrd + Add<&'a T, Output = T> + Sub<&'a T, Output = T> + Div<Output=T> + Mul<&'a T, Output = T>  + TryInto<U> + TryInto<usize> + TryFrom<usize>  + From<u8> + 'a,
          U: Default + Clone + Mul<Output = U> + From<u8> + Sum,
          &'a T: Sub<&'a T, Output = T> + Mul<&'a T, Output = T> + 'a,
          &'a U: Add<U,Output = U> + Sub<&'a U, Output = U> +'a
{
    type TypeTime = T;

    fn fold_signal(&self, signalb: &Signal<T,U>, sample_time: &Self::TypeTime) -> Option<Signal<T,U>>{
        self.fold_signal(signalb, sample_time)
    }
}

impl<'a,U> FoldableSignal for Signal<f64,U> 
where
    U: Default + Clone + Mul<Output = U> + From<u8> + Sum,
    &'a U: Add<U,Output = U> + Sub<&'a U, Output = U> +'a
{
    type TypeTime = f64;

    fn fold_signal(&self, signalb: &Signal<f64,U>, sample_time: &Self::TypeTime) -> Option<Signal<f64,U>>{
        None
    }
}


impl<T,U> Signal<T,U>
where T: Default + Clone,
      U: Default + Clone
{

    pub fn new(time_signal: Vec<T>, value_signal: Vec<U>) -> Signal<T,U>{
        assert_eq!(time_signal.len(), value_signal.len());
        Signal { 
            v: value_signal.iter().zip(time_signal)
                .map(|(x_val,t_val)| Sample{
                                                    t:t_val, 
                                                    x:(*x_val).clone()}
                ).collect()
        }
    }

    

    
    pub fn fold_signal<'a>(&'a self, signalb: &'a Signal<T,U>, sample_time: &'a T) -> Option<Signal<T,U>>
    where T: PartialOrd + Add<&'a T, Output = T> + Sub<&'a T, Output = T> + Div<Output=T> + Mul<&'a T, Output = T>  + TryInto<U> + TryInto<usize> + TryFrom<usize>  + From<u8> + 'a,
          U: Mul<Output = U> + From<u8> + Sum,
          &'a T: Sub<&'a T, Output = T> + Mul<&'a T, Output = T> + 'a,
          &'a U: Add<U,Output = U> + Sub<&'a U, Output = U> +'a
    {

        let t_min_siga = &self.v.first()?.t;
        let t_min_sigb = &signalb.v.first()?.t;

        if *sample_time == 0_u8.into(){
            return None;
        }

        let sig_a;
        let sig_b;
        if t_min_siga > t_min_sigb
        {
            let len_diff = ((t_min_siga - t_min_sigb)/(sample_time.clone())).try_into().ok()?;
            sig_a = [vec![0_u8.into();len_diff],self.resample(sample_time)].concat();
            sig_b = [signalb.resample(sample_time), vec![0_u8.into();sig_a.len()-len_diff]].concat();

        }else{
            let len_diff = ((t_min_sigb - t_min_siga)/(sample_time.clone())).try_into().ok()?;
            sig_a = self.resample(sample_time);
            sig_b = [vec![0_u8.into();len_diff], signalb.resample(sample_time), vec![0_u8.into();sig_a.len()]].concat();
        }
        
        let len_a = sig_a.len();
        let len_result = len_a+sig_b.len()-1;
        let mut result = vec![0_u8.into(); len_result];


        for (index, sample) in result.iter_mut().enumerate(){

            *sample = sig_a.iter()
                        .zip(sig_b.iter().rev().skip(len_result-index).take(len_a))
                        .map(|(a,b)| a.clone()*b.clone()).sum();

        }

        Some(Signal::new((0..len_result).into_iter().map(|i| match T::try_from(i){Ok(i_ok) => i_ok*sample_time, _=> 0_u8.into()}).collect(),
            result))
    }
    

    fn resample<'a>(&'a self, sampletime: &'a T) -> Vec<U>
    where T: PartialOrd + Add<&'a T, Output = T> + Sub<&'a T, Output = T> + Div<Output=T> + TryInto<U> +'a,
          U: Mul<Output = U> + From<u8>,
          &'a T: Sub<&'a T, Output = T> + 'a,
          &'a U: Add<U,Output = U> + Sub<&'a U, Output = U> + 'a
    {
        let first_sample = match self.v.first() {Some(s) => s, _ => return Vec::new()};

        let mut result = Vec::new();
        result.push(first_sample.x.clone());
        let mut new_t = first_sample.t.clone();

        for i in 1..self.v.len(){
            while new_t < self.v[i].t{
                new_t = new_t + sampletime;
                let new_x = &self.v[i-1].x+(&self.v[i].x-&self.v[i-1].x)*(((new_t.clone()-&self.v[i-1].t))/(&self.v[i].t-&self.v[i-1].t)).try_into().unwrap_or(1_u8.into());
                result.push(new_x);
            }
        }

        result
    }




}



fn fold_signal(signala: &Vec<f64>, signalb: &Vec<f64>) -> Vec<f64>
{

    let len_a = signala.len();
    let len_b = signalb.len();
    let len_result = len_a+len_b-1;
    let mut result = vec![0.0; len_result];


     for (index, sample) in result.iter_mut().enumerate(){

        *sample = signala.iter()
                    .zip([signalb.to_vec(), vec![0.0; len_a]].concat().iter().rev().skip(len_result-index).take(len_a))
                    .map(|(a,b)| *a*(*b)).sum();

    }

    result
}


#[cfg(test)]
mod tests {
    use crate::{fold_signal, Signal};
    use std::iter;

    #[test]
    fn add() {
        let signal = Signal::new(vec![0.0,1.0,2.0,3.0], vec![1,2,3,4]);

        let signal_b = Signal::new(vec![5.0,6.0,7.0,8.0], vec![1,0,0,0]);

        let signal_c = signal.fold_signal(&signal_b, &1.0).unwrap();

        assert_eq!(1, 2);
    }

    #[test]
    fn fold(){

        let a = vec![1.0,0.0,1.0];
        let b = vec![1.0,2.0,3.0,4.0,5.0];

        let c = fold_signal(&a, &b);

        assert_eq!(b, c);
    }


}