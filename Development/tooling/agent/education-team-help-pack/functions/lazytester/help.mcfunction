# Lazy Tester team commands — help only, no telemetry or gameplay changes.
# Triggered by /function help or /function lazytester/help in an operator QA world.
tellraw @s {"rawtext":[{"text":"§b§lLAZY TESTER - BANTUAN COMMAND§r"}]}
tellraw @s {"rawtext":[{"text":"§7Perintah berikut hanya tersedia jika modul tim terkait terpasang."}]}
tellraw @s {"rawtext":[{"text":"§eANALYTICS — modul analytics tim§r"}]}
tellraw @s {"rawtext":[{"text":"§f/scriptevent mivubi:get_analytics§r — ekspor analytics ke Content Log"}]}
tellraw @s {"rawtext":[{"text":"§eGAME RESULTS — modul scoring tim§r"}]}
tellraw @s {"rawtext":[{"text":"§f/scriptevent daigon:get_all_scoring§r — ekspor hasil permainan ke Content Log"}]}
tellraw @s {"rawtext":[{"text":"§eSKILLS PROBE — hanya world uji, admin§r"}]}
tellraw @s {"rawtext":[{"text":"§f/scriptevent probe:list§r — lihat daftar probe (Content Log)"}]}
tellraw @s {"rawtext":[{"text":"§f/scriptevent probe:run auto§r — jalankan tes di world uji saja"}]}
tellraw @s {"rawtext":[{"text":"§f/scriptevent probe:report§r — tampilkan laporan probe (Content Log)"}]}
tellraw @s {"rawtext":[{"text":"§eCOORDINATES — fitur bawaan Minecraft Education§r"}]}
tellraw @s {"rawtext":[{"text":"§fSettings > Show Coordinates. Jika Creator > Copy Coordinate UI tersedia: Ctrl+Alt+C, lalu Ctrl+V di Lazy Tester."}]}
tellraw @s {"rawtext":[{"text":"§7Saved Locations ada di aplikasi Lazy Tester; ini bukan perubahan HUD atau clipboard otomatis."}]}
tellraw @s {"rawtext":[{"text":"§7Help ini tidak memasang modul, menjalankan tes, atau mengirim data ke Codex."}]}
