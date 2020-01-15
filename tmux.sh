#! /usr/bin/env nix-shell
#! nix-shell -i bash -p gdb openocd cgdb tmux
SESSION=$USER

tmux -2 new-session -d -s $SESSION
tmux new-window -t $SESSION:1 -n 'ZC706'
tmux split-window -h
tmux select-pane -t 0
tmux send-keys "stty 115200 < /dev/ttyUSB1 && cat /dev/ttyUSB1" C-m
tmux select-pane -t 1
tmux send-keys "sleep 10 && cgdb zc706.elf -x openocd/gdb-zynq-commands" C-m
tmux split-window -v
tmux resize-pane -D 20
tmux send-keys "cd openocd && openocd -f zc706.cfg -c reset init" C-m

# Set default window
tmux select-window -t $SESSION:1

# Set focus on gdb
tmux select-pane -t $SESSION:.-

# Attach to session
tmux -2 attach-session -t $SESSION
