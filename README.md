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

# Running at least three subscribers simultaneously

## Screenshot of Terminal 3 simultaneous Subscribers + Publisher

![3 Subscribers Terminal](assets/images/Terminal%20of%203%20subscribers%20and%201%20publisher.png)

Dengan menjalankan **3 subscriber** secara bersamaan dan publisher dijalankan beberapa
kali, terlihat bahwa pemrosesan event terbagi secara otomatis di antara ketiga
subscriber. Tidak ada subscriber yang memproses event yang sama, RabbitMQ
mendistribusikan setiap pesan hanya ke **satu** consumer secara bergantian
(sesuai dengan algoritma *round-robin*). Misalnya, subscriber 1 memproses `Amir` dan `Dira`, subscriber 2 memproses `Budi` dan `Emir`, dan subscriber 3 memproses `Cica` dan seterusnya.

---

## Screenshot of RabbitMQ Queue Terkuras Lebih Cepat

![Multiple Subscribers Queue 1](assets/images/RabbitMQ%20multiple%20subsribers%20queue%201.png)
![Multiple Subscribers Queue 2](assets/images/RabbitMQ%20multiple%20subsribers%20queue%202.png)

Dibandingkan dengan skenario 1 subscriber, grafik **Queued messages** kini menunjukkan
spike yang jauh lebih kecil (maksimal ~10 pesan) dan antrian kembali ke 0 dengan
jauh lebih cepat. Terlihat pula pada **Global counts** bahwa `Consumers: 3` yang
mengkonfirmasi 3 subscriber aktif terhubung ke broker secara bersamaan.

---

## A little bit of reflection: Mengapa bisa seperti ini?

Karena setiap subscriber memiliki delay 1 detik per pesan, satu subscriber hanya
mampu mengonsumsi ~1 pesan/detik. Dengan 3 subscriber aktif, throughput konsumsi
efektif menjadi ~3 pesan/detik. Inilah yang disebut **horizontal scaling** pada
consumer. Menambah jumlah consumer adalah cara paling sederhana dan efektif untuk
mengatasi subscriber yang lambat tanpa mengubah kode sama sekali.

---

## Apa yang bisa diperbaiki dari kode publisher dan subscriber?

Melihat kembali kode keduanya, terdapat beberapa hal yang bisa ditingkatkan:

1. **Publisher tidak memiliki konfirmasi pengiriman.** Saat ini publisher menggunakan
   `_ = p.publish_event(...)` yang mengabaikan nilai return. Sebaiknya error ditangani
   dengan benar agar publisher tahu jika pengiriman gagal.

2. **Jumlah pesan publisher di-hardcode.** Publisher selalu mengirim tepat 5 pesan
   dengan nama yang tetap. Idealnya data ini bisa dibaca dari input atau file agar
   lebih fleksibel dan realistis.

3. **Subscriber tidak memiliki graceful shutdown.** Loop `loop {}` pada subscriber
   berjalan selamanya tanpa mekanisme untuk berhenti dengan bersih. Sebaiknya
   ditambahkan signal handler (misalnya menangkap `Ctrl+C`) agar koneksi ke broker
   ditutup dengan benar saat subscriber dihentikan.

4. **Delay di subscriber di-hardcode.** Nilai `1000ms` pada `thread::sleep` seharusnya
   bisa dikonfigurasi via environment variable, sehingga lebih mudah disesuaikan
   tanpa mengubah kode.

---

# Bonus: Running on Cloud

Untuk menyelesaikan tugas bonus, eksperimen di atas diulang kembali dengan RabbitMQ yang berjalan di cloud (Railway), bukan di mesin lokal. Publisher dan subscriber tetap dijalankan dari komputer lokal, tetapi keduanya terhubung ke broker RabbitMQ yang di-deploy pada cloud melalui URL koneksi `amqp://` yang disediakan oleh Railway.

## Screenshot of RabbitMQ Web UI on Cloud

![Running 3 subscribers on Cloud (RabbitMQ Web UI)](assets/images/Running%203%20subscribers%20on%20Cloud%20(rabbitmq%20web%20ui).png)

Pada tampilan RabbitMQ Management yang berjalan di cloud (Railway), terlihat bahwa broker berhasil diakses melalui URL `rabbitmq-web-ui-production-b12e.up.railway.app`. Global counts menunjukkan `Connections: 3`, `Exchanges: 11`, dan `Queues: 2`, yang
mengkonfirmasi bahwa ketiga subscriber berhasil terhubung ke broker cloud secara bersamaan. Node yang aktif adalah `rabbit@rabbitmq` yang berjalan di environment cloud Railway.

## Screenshot of Terminal Running 3 Subscribers + 1 Publisher on Cloud

![Running 3 subscribers on Cloud (Terminal)](assets/images/Running%203%20subscribers%20on%20Cloud%20(terminal).png)

Dari terminal, terlihat bahwa 1 publisher dan 3 subscribers dijalankan secara bersamaan, semuanya terhubung ke broker RabbitMQ cloud. Publisher mengirimkan 5 event (`Amir`, `Budi`, `Cica`, `Dira`, `Emir`) ke queue, dan ketiga subscriber menerima pesan secara terdistribusi (seimbang) menggunakan algoritma round-robin:

- **Subscriber 1** menerima: `Amir` (user_id: 1) dan `Dira` (user_id: 4)
- **Subscriber 2** menerima: `Budi` (user_id: 2) dan `Emir` (user_id: 5)
- **Subscriber 3** menerima: `Cica` (user_id: 3)

Hasil ini menunjukkan bahwa mekanisme horizontal scaling dan round-robin distribution pada RabbitMQ bekerja sama persis baik di lingkungan lokal maupun di cloud. Perbedaan utamanya hanyalah pada latensi jaringan yang sedikit lebih tinggi karena koneksi melewati internet, namun semua fungsionalitas event-driven architecture tetap berjalan dengan baik.