const pwm = document.getElementById("pwm");
const pwm_label = document.getElementById("pwm_label");

pwm_label.textContent = "PWM: " + pwm.value;

pwm.oninput = function() {
    pwm_label.textContent = "PWM: " + pwm.value;
    fetch("/pwm/" + pwm.value)
}