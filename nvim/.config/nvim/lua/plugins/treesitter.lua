return {
	{
		"nvim-treesitter/nvim-treesitter-textobjects",
		branch = "main",
		dependencies = { "nvim-treesitter/nvim-treesitter" },
		config = function()
			require("nvim-treesitter-textobjects").setup({
				select = {
					enable = true,
					lookahead = true,
					keymaps = {
						["af"] = { query = "@function.outer", desc = "outer function" },
						["if"] = { query = "@function.inner", desc = "inner function" },
						["aa"] = { query = "@parameter.outer", desc = "outer argument" },
						["ia"] = { query = "@parameter.inner", desc = "inner argument" },
						["ac"] = { query = "@class.outer", desc = "outer class" },
						["ic"] = { query = "@class.inner", desc = "inner class" },
						["igc"] = { query = "@comment.outer", desc = "inner comment" },
						["agc"] = { query = "@comment.outer", desc = "outer comment" },
					},
					selection_modes = {
						["@function.outer"] = "V",
					},
				},
			})
		end,
	},
	{
		"nvim-treesitter/nvim-treesitter",
		branch = "main",
		build = ":TSUpdate",
		dependencies = {
			{ "nvim-treesitter/nvim-treesitter-textobjects", branch = "main" },
		},
		lazy = false,
		init = function()
			vim.treesitter.language.register("bash", "zsh")
			vim.treesitter.language.register("bash", "tmux")
			vim.treesitter.language.register("java", "aidl")
			vim.filetype.add({
				filename = {
					["apple-app-site-association"] = "json",
				},
			})
			vim.api.nvim_create_augroup("filetype_overrides", { clear = true })
			vim.api.nvim_create_autocmd({ "BufRead", "BufNewFile" }, {
				group = "filetype_overrides",
				pattern = "*.pac",
				callback = function()
					vim.bo.filetype = "pac"
				end,
			})
			vim.treesitter.language.register("javascript", "pac")
		end,
		keys = {
			{ "<S-M-l>", "<cmd>TSTextobjectSwapNext @parameter.inner<CR>", desc = "Swap to previous parameter" },
		},
		config = function()
			require("nvim-treesitter").setup({
				auto_install = true,
				ensure_installed = { "latex", "regex" },
				highlight = { enable = true },
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
			})
		end,
	},
}
