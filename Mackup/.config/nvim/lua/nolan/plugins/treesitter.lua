return {
	"nvim-treesitter/nvim-treesitter",
	event = { "BufReadPre", "BufNewFile" },
	build = ":TSUpdate",
	dependencies = {
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
			incremental_selection = {
				enable = true,
				keymaps = {
					init_selection = "<CR>",
					node_incremental = "<CR>",
					scope_incremental = false,
					node_decremental = "<bs>",
				},
			},
			-- TODO: Find more useful textobjects
			textobjects = {
				select = {
					enable = true,
					-- Automatically jump forward to textobj, similar to targets.vim
					lookahead = true,
					keymaps = {
						["af"] = { query = "@function.outer", desc = "outer function" },
						["if"] = { query = "@function.inner", desc = "inner function" },

						["aa"] = { query = "@parameter.outer", desc = "outer argument" },
						["ia"] = { query = "@parameter.inner", desc = "inner argument" },
					},
					selection_modes = {
						["@function.outer"] = "V", -- linewise
					},
					include_surrounding_whitespace = true,
				},
			},
		})

		vim.treesitter.language.register("bash", "zsh")
		vim.treesitter.language.register("bash", "tmux")

		vim.keymap.set("n", "<leader>ti", "<cmd>InspectTree<CR>", { desc = "Inspect treesitter nodes" })
	end,
}
