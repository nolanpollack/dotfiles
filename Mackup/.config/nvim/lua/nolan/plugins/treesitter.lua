return {
	"nvim-treesitter/nvim-treesitter",
	event = { "BufReadPre", "BufNewFile" },
	build = ":TSUpdate",
	dependencies = {
		"windwp/nvim-ts-autotag",
		"nvim-treesitter/nvim-treesitter-textobjects",
	},
	config = function()
		-- import nvim-treesitter plugin
		local treesitter = require("nvim-treesitter.configs")

		-- configure treesitter
		treesitter.setup({ -- enable syntax highlighting
			-- automatically install missing parsers when entering buffer
			auto_install = true,
			highlight = {
				enable = true,
			},
			-- enable indentation
			indent = { enable = true },
			-- enable autotagging (w/ nvim-ts-autotag plugin)
			autotag = {
				enable = true,
			},
			incremental_selection = {
				enable = true,
				keymaps = {
					init_selection = "<CR>",
					node_incremental = "<CR>",
					scope_incremental = false,
					node_decremental = "<bs>",
				},
			},
			-- textobjects
			textobjects = {
				select = {
					enable = true,
					-- Automatically jump forward to textobj, similar to targets.vim
					lookahead = true,
					keymaps = {
						["af"] = { query = "@function.outer", desc = "outer function" },
						["if"] = { query = "@function.inner", desc = "inner function" },

						["a="] = { query = "@assignment.outer", desc = "outer assignment" },
						["i="] = { query = "@assignment.inner", desc = "inner assignment" },

						["ac"] = { query = "@comment.outer", desc = "outer comment" },
						["ic"] = { query = "@comment.inner", desc = "inner comment" },

						["aa"] = { query = "@parameter.outer", desc = "outer argument" },
						["ia"] = { query = "@parameter.inner", desc = "inner argument" },
					},
					selection_modes = {
						["@parameter.outer"] = "v", -- charwise
						["@function.outer"] = "V", -- linewise
					},
					include_surrounding_whitespace = true,
				},
			},
		})

		-- set keymap
		local keymap = vim.keymap -- for conciseness

		keymap.set("n", "<leader>ti", "<cmd>InspectTree<CR>", { desc = "Inspect treesitter nodes" })
	end,
}
