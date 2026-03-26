use std::process;

pub fn critical_error_handler(error: &str) -> ! {
    eprint!("Пиздец: {}\n", error);
    process::exit(1)
}

pub fn convert_to_f24_like(vec: &mut Vec<f32>) {
    for val in vec.iter_mut() {
        // Примерный диапазон и точность f24
        // Ограничиваем к числам в диапазоне [-8,388,608, 8,388,607] (24 бита: 1 знак, 23 дробные части)
        const MAX_F24: f32 = 8_388_608.0;
        const MIN_F24: f32 = -8_388_608.0;

        // Ограничиваем диапазон
        if *val > MAX_F24 {
            *val = MAX_F24;
        } else if *val < MIN_F24 {
            *val = MIN_F24;
        }

        // Урезаем точность до 24-битного представления
        *val = (*val * (1 << 23) as f32).round() / (1 << 23) as f32;
    }
}