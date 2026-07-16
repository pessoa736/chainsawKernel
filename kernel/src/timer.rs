
use embedded_time::Instant;
use embedded_time::Clock;
use embedded_time::duration;

pub struct TIMER;

pub static mut TIMER_TICKS: u64 = 0;

impl Clock for TIMER 
{
    type T = u64;

    const SCALING_FACTOR: embedded_time::rate::Fraction = 
        embedded_time::rate::Fraction::new(1, 1000); // 1/1000 segundos = 1ms

    fn try_now(&self) -> Result<embedded_time::Instant<Self>, embedded_time::clock::Error> {
        unsafe {
            Ok(embedded_time::Instant::new(TIMER_TICKS))
        }
    }
}


pub struct SubTimer {
    letreiro: Instant<TIMER>,
    duration: u64
}

impl SubTimer {
    pub fn start(relogio: &TIMER, duracao: u64) -> Self {
        Self {
            letreiro: relogio.try_now().unwrap(),
            duration: duracao
        }
    }

    pub fn expired(&mut self, relogio: &TIMER) -> bool {
        let agora = relogio.try_now().unwrap();
        agora.checked_duration_since(&self.letreiro)
        .map(|d| d.integer() >= self.duration)
        .unwrap_or(true);

        if agora.checked_duration_since(&self.letreiro).unwrap().integer() >= self.duration {
            self.letreiro = agora; 
            true
        } else {
            false
        }
    }

    pub fn percent(&self, relogio: &TIMER) -> f32
    {
        let agora = relogio.try_now().unwrap();

        let d = agora.checked_duration_since(&self.letreiro).unwrap().integer(); 
        
        if self.duration == 0 { return 0.0; }
        
        (d as f32 )/ (self.duration as f32)
    }
}



pub struct TimeInteration 
{
    _t: u64,
    _duration: Option<u64>,
    _close: bool
}

impl TimeInteration {
    pub fn new(duration: Option<u64>) -> Self
    {
        Self {
            _t: 0,
            _duration: duration,
            _close: false,
        }
    }

    pub fn loop_<F>(&mut self, mut _l: F)
        where  F: FnMut(&mut Self)
    {
        while self._duration.map_or(true, |dur| self._t < dur) {
            _l(self);
            
            if self._close { 
                break; 
            }
            
            self._t = self._t + 1;
        }
    }

    pub fn close(&mut self){ self._close = true; }
    
    pub fn reset(&mut self){ self._t = 0; }

    pub fn percent(&self) -> f32
    {
        (self._t as f32) / (self._duration.unwrap() as f32)
    }
}