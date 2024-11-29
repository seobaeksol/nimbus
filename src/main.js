const { invoke } = window.__TAURI__.core;
const { WebviewWindow } = window.__TAURI__.webviewWindow
const { getCurrentWebviewWindow } = window.__TAURI__.webviewWindow
const { emitTo } = window.__TAURI__.event

let greetInputEl;
let greetMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

greetInputEl = document.querySelector("#greet-input");
greetMsgEl = document.querySelector("#greet-msg");

document.querySelector("#greet-form").addEventListener("submit", (e) => {
  e.preventDefault();
  greet();
});

const sendMessageForm = document.querySelector('#send-message-form')
const sendMessageEl = document.querySelector('#send-message')
const sendLabelEl = document.querySelector('#send-label')
sendMessageForm.addEventListener('submit', (e) => {
  e.preventDefault()
  console.log(sendLabelEl.value)
  console.log(sendMessageEl.value)

  emitTo(sendLabelEl.value, 'message', sendMessageEl.value)
})

const newWindowForm = document.querySelector('#new-window-form')
const newLabelEl = document.querySelector('#new-label')
const newTitleEl = document.querySelector('#new-title')
newWindowForm.addEventListener('submit', (e) => {
  e.preventDefault()

  new WebviewWindow(newLabelEl.value, {
    title: newTitleEl.value
  })
})


window.addEventListener("DOMContentLoaded", () => {

  const messagesView = document.querySelector('#messages-view')
  const currentWindow = getCurrentWebviewWindow()
  currentWindow.listen('message', (event) => {
    const time = new Date().toLocaleTimeString()
    messagesView.textContent = `${messagesView.textContent}\n[${time}] ${event.payload}`
  })
});
