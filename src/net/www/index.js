const x = document.getElementById("x");
const y = document.getElementById("y");
const z = document.getElementById("z");
const x_label = document.getElementById("x_label");
const y_label = document.getElementById("y_label");
const z_label = document.getElementById("z_label");

x_label.textContent = "X: " + x.value / 100;
y_label.textContent = "Y: " + y.value / 100;
z_label.textContent = "Z: " + z.value / 100;

x.oninput = function() {
    x_label.textContent = "X: " + x.value / 100;
    fetch("/pos/" + x.value + "/" + y.value + "/" + z.value)
}

y.oninput = function() {
    y_label.textContent = "Y: " + y.value / 100;
    fetch("/pos/" + x.value + "/" + y.value + "/" + z.value)
}

z.oninput = function() {
    z_label.textContent = "Z: " + z.value / 100;
    fetch("/pos/" + x.value + "/" + y.value + "/" + z.value)
}