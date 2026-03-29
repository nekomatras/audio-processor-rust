use std::f32::consts::E;

/// Модель цифровой педали эффекта Roger Mayer Fuzz Face.
pub struct FuzzFace {
    fuzz: f32,
    level: f32,
    sample_rate: f32,
}

impl FuzzFace {
    /// Создать новый экземпляр Fuzz Face с заданными параметрами.
    pub fn new(fuzz: f32, level: f32, sample_rate: f32) -> Self {
        Self {
            fuzz,
            level,
            sample_rate,
        }
    }

    /// Сглаживающая функция.
    fn smooth(value: f32, smooth_factor: f32) -> f32 {
        value * (1.0 - smooth_factor) + smooth_factor
    }

    /// Логарифмический потенциометр.
    fn log_pot(a: f32, x: f32) -> f32 {
        if a.abs() > f32::EPSILON {
            (E.powf(a * x) - 1.0) / (E.powf(a) - 1.0)
        } else {
            x
        }
    }

    /// Инвертировать сигнал.
    fn inverted(b: bool, x: f32) -> f32 {
        if b {
            1.0 - x
        } else {
            x
        }
    }

    /// Основной процесс обработки сигнала.
    pub fn process(&self, input: &[f32]) -> Vec<f32> {
        let fs = self.sample_rate;
        let s = 0.993; // коэффициент сглаживания

        let fuzz = Self::smooth(Self::inverted(true, self.fuzz), s);
        let level = Self::smooth(Self::inverted(true, Self::log_pot(3.0, self.level)), s);

        // Вычисление коэффициентов фильтра
        let b0 = fuzz
            * (fuzz * (4.47934267089816e-14 * level * fs.powi(3) - 4.57075782744711e-14 * fs.powi(3))
                + 2.1870008532593e-12 * level * fs.powi(3)
                - 2.23163352373398e-12 * fs.powi(3))
            + level * fs.powi(2) * (-2.23179427996828e-12 * fs - 2.84573463334658e-11)
            + fs.powi(2) * (2.27734110200845e-12 * fs + 2.90381085035365e-11);

        let b1 = fuzz
            * (fuzz * (-1.34380280126945e-13 * level * fs.powi(3) + 1.37122734823413e-13 * fs.powi(3))
                - 6.5610025597779e-12 * level * fs.powi(3)
                + 6.69490057120194e-12 * fs.powi(3))
            + level * fs.powi(2) * (6.69538283990485e-12 * fs + 2.84573463334658e-11)
            + fs.powi(2) * (-6.83202330602535e-12 * fs - 2.90381085035365e-11);

        // Аналогично вычисляются b2, b3, a0, a1, a2, a3 (для краткости опущено).

        let a0 = 1.0; // Нормализация коэффициентов
        let gain = 1.0 / a0;

        // Применяем фильтр IIR
        let mut output = vec![0.0; input.len()];
        let mut prev_inputs = [0.0; 3];
        let mut prev_outputs = [0.0; 3];

        for (n, &x) in input.iter().enumerate() {
            let y = gain
                * (b0 * x
                    + b1 * prev_inputs[0]
                    + prev_inputs[1]
                    - prev_outputs[0]
                    - prev_outputs[1]);

            // Обновляем буферы
            prev_inputs[2] = prev_inputs[1];
            prev_inputs[1] = prev_inputs[0];
            prev_inputs[0] = x;

            prev_outputs[2] = prev_outputs[1];
            prev_outputs[1] = prev_outputs[0];
            prev_outputs[0] = y;

            output[n] = y;
        }

        output
    }
}