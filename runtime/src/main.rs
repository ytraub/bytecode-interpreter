extern crate ev3dev_lang_rust;

use ev3dev_lang_rust::motors::{LargeMotor, MotorPort};
use ev3dev_lang_rust::sensors::{ColorSensor, SensorPort};
use ev3dev_lang_rust::Ev3Result;

fn main() -> Ev3Result<()> {
    // Get large motor on port outA.
    let large_motor = LargeMotor::get(MotorPort::OutA)?;

    // Set command "run-direct".
    large_motor.run_direct()?;

    // Run motor.
    large_motor.set_duty_cycle_sp(50)?;

    // Find color sensor. Always returns the first recognized one.
    let color_sensor = ColorSensor::get(SensorPort::In1)?;

    // Switch to rgb mode.
    color_sensor.set_mode_rgb_raw()?;

    // Get current rgb color tuple.
    println!("Current rgb color: {:?}", color_sensor.get_rgb()?);

    large_motor.stop()?;

    Ok(())
}
