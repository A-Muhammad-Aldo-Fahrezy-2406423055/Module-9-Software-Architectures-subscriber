# Understanding Subscriber and Message Broker

## a. What is AMQP?

AMQP (*Advanced Message Queuing Protocol*) adalah protokol komunikasi jaringan berbasis standar terbuka yang digunakan untuk *message-oriented middleware*. Protokol ini memungkinkan aplikasi yang berbeda (bahkan yang ditulis dalam bahasa pemrograman berbeda) untuk saling bertukar pesan secara andal melalui sebuah *message broker*. AMQP mendefinisikan cara pesan dikirim, dirutekan, dan diantrekan, sehingga menjamin interoperabilitas antar sistem. RabbitMQ adalah salah satu implementasi populer dari protokol AMQP ini.

## b. Apa arti `guest:guest@localhost:5672`?

URL `amqp://guest:guest@localhost:5672` dapat diuraikan sebagai berikut:

- **`guest` pertama** adalah *username* yang digunakan untuk autentikasi ke RabbitMQ. Dalam hal ini, `guest` adalah akun default yang disediakan oleh RabbitMQ.
- **`guest` kedua** adalah *password* dari username tersebut. Secara default, RabbitMQ menetapkan password `guest` untuk user `guest`.
- **`localhost:5672`** adalah alamat dan port dari *message broker* RabbitMQ yang sedang berjalan. `localhost` berarti broker berjalan di mesin yang sama dengan program subscriber, sedangkan `5672` adalah port default yang digunakan RabbitMQ untuk menerima koneksi AMQP.