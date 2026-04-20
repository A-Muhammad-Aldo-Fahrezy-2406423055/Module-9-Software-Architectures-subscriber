# Understanding Subscriber and Message Broker

## a. What is AMQP?

AMQP (*Advanced Message Queuing Protocol*) adalah protokol komunikasi jaringan berbasis standar terbuka yang digunakan untuk *message-oriented middleware*. Protokol ini memungkinkan aplikasi yang berbeda (bahkan yang ditulis dalam bahasa pemrograman berbeda) untuk saling bertukar pesan secara andal melalui sebuah *message broker*. AMQP mendefinisikan cara pesan dikirim, dirutekan, dan diantrekan, sehingga menjamin interoperabilitas antar sistem. RabbitMQ adalah salah satu implementasi populer dari protokol AMQP ini.

## b. Apa arti `guest:guest@localhost:5672`?

URL `amqp://guest:guest@localhost:5672` dapat diuraikan sebagai berikut:

- **`guest` pertama** adalah *username* yang digunakan untuk autentikasi ke RabbitMQ. Dalam hal ini, `guest` adalah akun default yang disediakan oleh RabbitMQ.
- **`guest` kedua** adalah *password* dari username tersebut. Secara default, RabbitMQ menetapkan password `guest` untuk user `guest`.
- **`localhost:5672`** adalah alamat dan port dari *message broker* RabbitMQ yang sedang berjalan. `localhost` berarti broker berjalan di mesin yang sama dengan program subscriber, sedangkan `5672` adalah port default yang digunakan RabbitMQ untuk menerima koneksi AMQP.

# Simulating Slow Subscriber

Setelah mengaktifkan `thread::sleep(ten_millis)` (delay 1 detik per pesan) pada
`subscriber/src/main.rs`, subscriber kini memproses setiap event secara jauh lebih
lambat dibandingkan kecepatan publisher mengirimnya.

## Screenshot of Queue Menumpuk

![Queue Buildup 1](assets/images/RabbitMQ%20queue%201.png)
![Queue Buildup 2](assets/images/RabbitMQ%20queue%202.png)

Ketika publisher dijalankan berkali-kali secara cepat dengan `cargo run`, terlihat
pada grafik **Queued messages** bahwa jumlah pesan yang antri terus meningkat hingga
mencapai **Total: 96** (Unacked: 96). Ini terjadi karena subscriber hanya mampu
memproses **1 pesan per detik**, sedangkan publisher bisa mengirim 5 pesan dalam
waktu kurang dari 1 detik.

## Mengapa total queue mencapai 96?

Publisher dijalankan sebanyak kurang lebih **19–21 kali** secara cepat (19 × 5 = 95,
atau 21 × 5 = 105 pesan); Dari grafik kita bisa melihat bahwa nilai queued messages melebihi 100. Namun, karena subscriber sudah sempat memproses beberapa pesan di sela-sela pengiriman, angka yang terekam di broker saat dipantau adalah sekitar **96 pesan**. Setiap 1 detik, subscriber hanya menyelesaikan 1 pesan, sehingga antrian menumpuk jauh lebih cepat daripada dikonsumsi.

Ini membuktikan konsep **back-pressure** dalam event-driven architecture: message
broker bertindak sebagai buffer dan publisher tidak pernah crash meskipun subscriber lambat, karena semua pesan ditampung di queue terlebih dahulu.

---

## Screenshot of Queue Perlahan Terkuras

![Queue Draining 1](assets/images/RabbitMQ%20queue%203.png)
![Queue Draining 2](assets/images/RabbitMQ%20queue%204.png)

Setelah publisher berhenti dijalankan, grafik **Queued messages** menunjukkan garis
merah yang turun secara konstan dan linear. Grafik **Message rates** memperlihatkan
*Consumer ack* stabil di sekitar **1.0/s** yang sesuai persis dengan delay 1 detik
yang diset pada `thread::sleep`. Queue akhirnya mencapai **Total: 0** ketika semua
pesan berhasil diproses satu per satu oleh subscriber.

Pola penurunan yang lambat dan linear ini adalah bukti nyata dari subscriber yang
lambat, berbeda dengan subscriber normal yang langsung menguras queue hampir
seketika.