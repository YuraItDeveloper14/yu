# Безпека · Security

## Українською

**Підтримувані версії.** Виправлення отримує 1.x. Версії до 1.0 не підтримуються.

**Як повідомити про вразливість.** Приватно: вкладка **Security** цього репозиторію →
**Report a vulnerability**. Не відкривай публічний issue. Опиши, що можна зробити й як це повторити.

**Що варто знати про Yu:**

- Команда `yu` виконує лише програму, яку ти їй дав, читає файли, які ти назвав, і записує малюнок
  туди, куди скажеш через `--svg`.
- Yu Studio виконує програму у твоєму браузері й нікуди її не надсилає. Посилання «Поділитися» тримає
  програму після `#` в адресі, а цю частину адреси браузер серверу не передає.
- Сайт надсилає Content-Security-Policy та інші заголовки безпеки.

## In English

**Supported versions.** 1.x gets fixes. Versions before 1.0 are not supported.

**Reporting a vulnerability.** Privately: this repository's **Security** tab →
**Report a vulnerability**. Please do not open a public issue. Say what can be done and how to
reproduce it.

**Good to know about Yu:**

- The `yu` command runs only the program you give it, reads the files you name, and writes a
  picture where `--svg` says.
- Yu Studio runs a program in your browser and sends it nowhere. A Share link keeps the program
  after `#` in the address, and browsers do not send that part to the server.
- The site sends a Content-Security-Policy and other security headers.
