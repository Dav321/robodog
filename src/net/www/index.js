const upper_servo = document.getElementById("upper_servo");
const lower_servo = document.getElementById("lower_servo");
const upper_servo_label = document.getElementById("upper_servo_label");
const lower_servo_label = document.getElementById("lower_servo_label");

upper_servo_label.textContent = "Upper servo: " + upper_servo.value;
lower_servo_label.textContent = "Lower servo: " + lower_servo.value;

upper_servo.oninput = function() {
    upper_servo_label.textContent = "Upper servo: " + upper_servo.value;
    fetch("/upper_servo/" + upper_servo.value)
}

lower_servo.oninput = function() {
    lower_servo_label.textContent = "Lower servo: " + lower_servo.value;
    fetch("/lower_servo/" + lower_servo.value)
}