-- TODO: Ensure this is configured correctly for neovim 11. Is there any way to ensure it's async?
return {
	"nvim-treesitter/nvim-treesitter",
	event = { "BufReadPre", "BufNewFile" },
	build = ":TSUpdate",
	dependencies = {
		"nvim-treesitter/nvim-treesitter-textobjects",
	},
	lazy = vim.fn.argc(-1) == 0, -- load treesitter early when opening a file from the cmdline
	init = function()
		vim.treesitter.language.register("bash", "zsh")
		vim.treesitter.language.register("bash", "tmux")
		vim.treesitter.language.register("java", "aidl")
        vim.filetype.add({
          filename = {
            ['apple-app-site-association'] = 'json',
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
        {"<S-M-l>", "<cmd>TSTextobjectSwapNext @parameter.inner<CR>", desc = "Swap to previous parameter"},
	},
	main = "nvim-treesitter.configs",
	opts = {
		-- automatically install missing parsers when entering buffer
		auto_install = true,
		highlight = {
			enable = true,
		},
        ensure_installed = {
            "latex",
            "regex"
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

					["ac"] = { query = "@class.outer", desc = "outer class" },
					["ic"] = { query = "@class.inner", desc = "inner class" },

					["igc"] = { query = "@comment.outer", desc = "inner comment" },
					["agc"] = { query = "@comment.outer", desc = "outer comment" },
				},
				selection_modes = {
					["@function.outer"] = "V", -- linewise
				},
			},
		},
	},
}
