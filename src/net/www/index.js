const x = document.getElementById("x");
const y = document.getElementById("y");
const x_label = document.getElementById("x_label");
const y_label = document.getElementById("y_label");

x_label.textContent = "X: " + x.value;
y_label.textContent = "Y: " + y.value;

x.oninput = function() {
    x_label.textContent = "X: " + x.value;
    fetch("/pos/" + x.value + "/" + y.value)
}

y.oninput = function() {
    y_label.textContent = "Y: " + y.value;
    fetch("/pos/" + x.value + "/" + y.value)
}