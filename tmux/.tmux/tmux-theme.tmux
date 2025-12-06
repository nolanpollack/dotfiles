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

FLAMINGO=#f2cdcd
MAUVE='#cba6f7'
MAROON=#eba0ac
PEACH=#fab387
GREEN=#a6e3a1
SAPPHIRE=#74c7ec
BLUE=#89b4fa #236
LAVENDER=#b4befe
OVERLAY0=#6c7086
SURFACE1=#45475a #237
BASE="#1e1e2e"
MANTLE=#181825
CRUST=#11111b

FG="#f38ba8"

# Status options
tmux_set status-interval 1
tmux_set status on

# Basic status bar colors
tmux_set status-fg "$FG"
tmux_set status-bg "$BASE"
tmux_set status-attr none

PREFIX_FG=$MANTLE
PREFIX_BG=$BLUE

SESSION_FG=$MANTLE
SESSION_BG=$MAUVE

# tmux-prefix-highlight
tmux_set @prefix_highlight_fg "$PREFIX_BG"
tmux_set @prefix_highlight_bg "$PREFIX_FG"
tmux_set @prefix_highlight_show_copy_mode 'on'
tmux_set @prefix_highlight_copy_mode_attr "fg=$PREFIX_BG,bg=$PREFIX_FG,bold" # TODO
tmux_set @prefix_highlight_output_prefix ""
tmux_set @prefix_highlight_output_suffix " #[fg=$PREFIX_FG]#[bg=$SESSION_BG]$rarrow"

#     
# Left side of status bar
tmux_set status-left-bg "$BASE"
tmux_set status-left-fg "$BASE"
tmux_set status-left-length 150
LS="#{prefix_highlight}\
#[fg=$BASE,bg=$SESSION_BG,bold] $session_icon \
#[fg=$SESSION_BG,bg=$SESSION_FG]$rarrow \
#[fg=$SESSION_BG,bg=$SESSION_FG]#S \
#[fg=$SESSION_FG,bg=$BASE]$rarrow"
tmux_set status-left "$LS"

tmux_set status-right ""

# Window status format
tmux_set window-status-format "#[fg=$BASE,bg=""$FLAMINGO""]""$rarrow\
#[fg=""$BASE"",bg=$FLAMINGO]#I""\
#[fg=""$FLAMINGO"",bg=""$SURFACE1""]$rarrow #W \
#[fg=$SURFACE1,bg=$BASE]$rarrow"

CURRENT_HIGHLIGHT=$PEACH
CURRENT_BG=$FLAMINGO
CURRENT_TEXT=$BASE
tmux_set window-status-current-format "#[fg=$BASE,bg=$CURRENT_HIGHLIGHT]$rarrow\
#[fg=$BASE,bg=$CURRENT_HIGHLIGHT,bold]#I\
#[fg=$CURRENT_HIGHLIGHT,bg=$CURRENT_BG,bold]$rarrow\
#[fg=$CURRENT_TEXT,bg=$CURRENT_BG,bold] #W \
#[fg=$CURRENT_BG,bg=$BASE,nobold]$rarrow"

# Window status style
tmux_set window-status-style          "fg=$FLAMINGO,bg=$BASE,none"
tmux_set window-status-last-style     "fg=$FLAMINGO,bg=$BASE,bold"
tmux_set window-status-activity-style "fg=$FLAMINGO,bg=$BASE,bold"

# Window separator
tmux_set window-status-separator ""

# Pane border
tmux_set pane-border-style "fg=$MANTLE,bg=default"

# Active pane border
tmux_set pane-active-border-style "fg=$MAUVE,bg=default"

# Pane number indicator
tmux_set display-panes-colour "$MANTLE"
tmux_set display-panes-active-colour "$BLUE"

# Clock mode
tmux_set clock-mode-colour "$MAUVE"
tmux_set clock-mode-style 24

# Message
tmux_set message-style "fg=$MAUVE,bg=$BASE"

# Command message
tmux_set message-command-style "fg=$MAUVE,bg=$BASE"

# Copy mode highlight
tmux_set mode-style "bg=$MAUVE,fg=$MANTLE"
