return {
	"yetone/avante.nvim",
	event = "VeryLazy",
	lazy = false,
	version = false, -- set this if you want to always pull the latest change
	init = function()
		vim.treesitter.language.register("markdown", "Avante")
	end,
	opts = {
		provider = "claude",
		behavior = {
			auto_suggestions = false,
		},
		windows = {
            width = 40,
			input = {
				height = 2,
			},
			ask = {
				floating = true,
			},
		},
        mappings = {
            diff = {
                ours = "<leader>co",
                theirs = "<leader>ct",
                all_theirs = "<leader>ca",
                both = "<leader>cb", -- What does this do?
                cursor = "<leader>cc",
            }
        }
	},
	-- if you want to build from source then do `make BUILD_FROM_SOURCE=true`
	build = "make",
	dependencies = {
		"stevearc/dressing.nvim",
		"nvim-lua/plenary.nvim",
		"MunifTanjim/nui.nvim",
		--- The below dependencies are optional,
		"hrsh7th/nvim-cmp", -- autocompletion for avante commands and mentions
		"echasnovski/mini.icons",
		"OXY2DEV/markview.nvim",
	},
}
