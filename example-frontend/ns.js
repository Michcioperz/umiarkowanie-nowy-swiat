const nam = document.getElementById("nam");
const tim = document.getElementById("tim");
const socket = new Paho.MQTT.Client("192.168.2.1", 1280, "nsu");
const button = document.getElementById("enNot");

socket.onMessageArrived = (message) => {
  nam.innerText = message.payloadString;
  tim.innerText = new Date().toLocaleTimeString();
  if (Notification.permission == "granted") {
    new Notification(message.payloadString, {silent: true});
  }
};

function onConnect() {
  socket.subscribe("radiopush/nowyswiat/StreamTitle");
}

socket.onConnectionLost = evt => {
  socket.connect({onSuccess: onConnect});
};

socket.onConnectionLost(); 

function onAgree() {
  if (Notification.permission == "granted") {
    button.parentNode.removeChild(button);
  }
}

button.addEventListener("click", () => {
  Notification.requestPermission().then(onAgree);
});
onAgree();
