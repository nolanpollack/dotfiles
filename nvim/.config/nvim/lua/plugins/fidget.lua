return {
	"j-hui/fidget.nvim",
	cond = not vim.g.scrollback_mode,
	opts = {
		progress = {
			display = {
				done_icon = "✓",
			},
		},
		notification = {
			window = {
				winblend = 0,
			},
			override_vim_notify = true,
		},
	},
}
