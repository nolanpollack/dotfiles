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
			},
			custom_highlights = function(colors)
				return {
					GitGraphHash = { fg = colors.mauve },
					GitGraphTimestamp = { fg = colors.peach },
					GitGraphAuthor = { fg = colors.blue },
					GitGraphBranchName = { fg = colors.pink },
					GitGraphBranchTag = { fg = colors.red },
					GitGraphBranchMsg = { fg = colors.text },
					GitGraphBranch1 = { fg = colors.red },
					GitGraphBranch2 = { fg = colors.green },
					GitGraphBranch3 = { fg = colors.blue },
					GitGraphBranch4 = { fg = colors.yellow },
					GitGraphBranch5 = { fg = colors.mauve },
				}
			end,
		})

		vim.cmd.colorscheme("catppuccin")
	end,
}
