return {
	"catppuccin/nvim",
	name = "catppuccin",
	priority = 1000,
	config = function()
		require("catppuccin").setup({
			flavour = "mocha",
			integrations = {
				navic = {
					enabled = true,
				},
				fidget = true,
				noice = true,
				diffview = true,
			},
			highlight_overrides = {
				mocha = function(colors)
					return {
						CmpItemMenu = { fg = colors.subtext0 },
						Pmenu = { bg = "NONE" },
					}
				end,
			},
		})

		vim.cmd.colorscheme("catppuccin")
	end,
}
