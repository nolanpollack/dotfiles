mod plugin;
mod zellij;

use plugin::State;
use zellij_tile::prelude::*;

register_plugin!(State);
