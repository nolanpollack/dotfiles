#!/usr/bin/env bash
#===============================================================================
#   Author: Wenxuan
#    Email: wenxuangm@gmail.com
#  Created: 2018-04-05 17:37
#===============================================================================

# $1: option
# $2: default value
tmux_get() {
    local value="$(tmux show -gqv "$1")"
    [ -n "$value" ] && echo "$value" || echo "$2"
}

# $1: option
# $2: value
tmux_set() {
    tmux set-option -gq "$1" "$2"
}

# Options
rarrow=$(tmux_get '@tmux_power_right_arrow_icon' '')
session_icon="$(tmux_get '@tmux_power_session_icon' '')"

MAUVE='#cba6f7'
BASE=#262626 #235
BLUE=#89b4fa #236
FLAMINGO=#f2cdcd
SURFACE1=#45475a #237

FG="#f38ba8"
BG="#1e1e2e"

# Status options
tmux_set status-interval 1
tmux_set status on

# Basic status bar colors
tmux_set status-fg "$FG"
tmux_set status-bg "$BG"
tmux_set status-attr none

# tmux-prefix-highlight
tmux_set @prefix_highlight_fg "$BG"
tmux_set @prefix_highlight_bg "$BLUE"
tmux_set @prefix_highlight_show_copy_mode 'on'
tmux_set @prefix_highlight_copy_mode_attr "fg=$MAUVE,bg=$BG,bold"
tmux_set @prefix_highlight_output_prefix ""
tmux_set @prefix_highlight_output_suffix " #[fg=$BLUE]#[bg=$MAUVE]$rarrow"

#     
# Left side of status bar
tmux_set status-left-bg "$BASE"
tmux_set status-left-fg "$BASE"
tmux_set status-left-length 150
LS="#{prefix_highlight}#[fg=$BASE,bg=$MAUVE,bold] $session_icon #S #[fg=$MAUVE,bg=$BG]$rarrow"
tmux_set status-left "$LS"

tmux_set status-right ""

# Window status format
tmux_set window-status-format         "#[fg=$BG,bg=""$SURFACE1""]""$rarrow""#[fg=""$FLAMINGO"",bg=""$SURFACE1""] #I #W #[fg=$SURFACE1,bg=$BG]$rarrow"
tmux_set window-status-current-format "#[fg=$BG,bg=$FLAMINGO]$rarrow#[fg=$BG,bg=$FLAMINGO,bold] #I #W #[fg=$FLAMINGO,bg=$BG,nobold]$rarrow"

# Window status style
tmux_set window-status-style          "fg=$FLAMINGO,bg=$BG,none"
tmux_set window-status-last-style     "fg=$FLAMINGO,bg=$BG,bold"
tmux_set window-status-activity-style "fg=$FLAMINGO,bg=$BG,bold"

# Window separator
tmux_set window-status-separator ""

# Pane border
tmux_set pane-border-style "fg=$MAUVE,bg=default"

# Active pane border
tmux_set pane-active-border-style "fg=$MAUVE,bg=default"

# Pane number indicator
tmux_set display-panes-colour "$FLAMINGO"
tmux_set display-panes-active-colour "$MAUVE"

# Clock mode
tmux_set clock-mode-colour "$MAUVE"
tmux_set clock-mode-style 24

# Message
tmux_set message-style "fg=$MAUVE,bg=$BG"

# Command message
tmux_set message-command-style "fg=$MAUVE,bg=$BG"

# Copy mode highlight
tmux_set mode-style "bg=$MAUVE,fg=$FG"
