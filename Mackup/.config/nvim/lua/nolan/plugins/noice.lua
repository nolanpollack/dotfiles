return {
	"folke/noice.nvim",
	event = "VeryLazy",
	dependencies = {
		"MunifTanjim/nui.nvim",
	},
	opts = {
		messages = {
			enabled = false,
		},
		popupmenu = {
			enabled = false,
		},
		notify = {
			enabled = false,
		},
		lsp = {
			progress = {
				enabled = false,
			},
			hover = {
				enabled = true,
			},
			signature = {
				enabled = true,
			},
			message = {
				enabled = false,
			},
		},
	},
}
