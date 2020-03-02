All-to-all messaging daemon
===========================

WARNING: a2amd is probably insecure, definitely unoptimized, etc. so use carefully.

Proof of concept instant messaging daemon inspired by [PyBitmessage](https://github.com/Bitmessage/PyBitmessage).
Every daemon sends/receives all messages to/from every other daemon.
Messages are prioritized by their proof of work with newer and smaller messages replacing older and larger ones in a2amd's memory-only database.


Installation
------------

	git clone https://github.com/iljah/a2amd
	cd a2amd
	cargo build


Usage
-----

After installation, in one terminal run...

	./target/debug/a2amd 127.0.0.1:1234 200 info

...and in another, send a2amd some messages for storage

	echo set a 1970-01-01T00:00:00 hello_old_world | socat -t 2 - TCP4:127.0.0.1:1234
	echo set c 2020-01-01T00:00:00 hello_new_world | socat -t 2 - TCP4:127.0.0.1:1234

`./target/debug/create_message` can be used to create messages automatically, including the required proof of work

	$ ./target/debug/create_message hello_world_4 4
	set BR 2020-03-02T09:13:47 hello_world_4
	$ ./target/debug/create_message hello_world_8 8
	set rFZ 2020-03-02T09:13:49 hello_world_8
	$ ./target/debug/create_message hello_world_12 12
	set CpG6 2020-03-02T09:14:00 hello_world_12
	$ ./target/debug/create_message hello_world_16 16
	set DoG52 2020-03-02T09:14:47 hello_world_16
	$ ./target/debug/create_message hello_world_20 20
	set vtBLnW 2020-03-02T09:14:17 hello_world_20

Sending above to a2amd will show the messages' hashes getting smaller in a2amd's output due to larger proof of work

	replacing 00752C7A03... with 00104F469D...
	replacing 00104F469D... with 000BA87071...
	replacing 000BA87071... with 0000C2B5E2...
	replacing 0000C2B5E2... with 000000C178...

`sync.sh` can be used to transfer messages between two daemons

	./sync.sh 127.0.0.1:1234 127.0.0.1:1235

Interactive use is possible with `socat`

	$ socat -t 2 - TCP4:127.0.0.1:1234
	>help
	help, list, get, set
	>help get
	get hash: return message with given hash (in hex, without leading 0x)
	>help set
	set nonce timestamp payload: add given message, timestamp must be in YYYY-MM-DDThh:mm:ss format
	>set BR 2020-03-02T09:13:47 hello_world_4
	ok
	>list
	00752C7A036DC8A58FADA0696904B0DC1EE76684FCACD6925F9B6114CE7AB4715DEFC833FC90CB447C01677E30E2BAA29BD0B8F70176AA8D748F3F5AB1C7511A 
	>get 00752C7A036DC8A58FADA0696904B0DC1EE76684FCACD6925F9B6114CE7AB4715DEFC833FC90CB447C01677E30E2BAA29BD0B8F70176AA8D748F3F5AB1C7511A
	BR 2020-03-02T09:13:47 hello_world_4
