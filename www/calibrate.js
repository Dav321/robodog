const pwm = document.getElementById("pwm");
const pwm_label = document.getElementById("pwm_label");

pwm_label.textContent = "PWM: " + pwm.value;

pwm.oninput = function() {
    let leg = document.querySelector('input[name="leg"]:checked').value;
    let motor = document.querySelector('input[name="motor"]:checked').value;
    let index = (leg * 3) + (motor * 1);
    pwm_label.textContent = "PWM: " + (pwm.value / 6666.66);
    fetch("/pwm/" + index + "/" + pwm.value)
}