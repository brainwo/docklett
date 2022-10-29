#!/bin/zsh

bspc rule -l | grep "Docklett" || bspc rule -a "Docklett" monitor="HDMI-1" follow=off focus=off
kitty @ set-tab-title " cargo" | cargo watch -x 'run'
