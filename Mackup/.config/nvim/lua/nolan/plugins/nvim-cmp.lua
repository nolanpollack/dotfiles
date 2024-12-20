return {
	"hrsh7th/nvim-cmp",
	event = { "InsertEnter", "CmdlineEnter" },
	dependencies = {
		"hrsh7th/cmp-buffer", -- source for text in buffer
		"hrsh7th/cmp-path", -- source for file system paths
		"hrsh7th/cmp-cmdline", -- source for command-line completions
		{
			"L3MON4D3/LuaSnip",
			-- follow latest release.
			version = "v2.*", -- Replace <CurrentMajor> by the latest released major (first number of latest release)
			-- install jsregexp (optional!).
			build = "make install_jsregexp",
		},
		"saadparwaiz1/cmp_luasnip", -- for autocompletion
		"rafamadriz/friendly-snippets", -- useful snippets
		"onsails/lspkind.nvim", -- vs-code like pictograms
		"luckasRanarison/tailwind-tools.nvim", -- tailwind highlighting
	},
	opts = function()
		local cmp = require("cmp")
		local luasnip = require("luasnip")
		local lspkind = require("lspkind")
		local copilot = require("copilot.suggestion")
        local neotab = require("neotab");

		-- loads vscode style snippets from installed plugins (e.g. friendly-snippets)
		require("luasnip.loaders.from_vscode").lazy_load()

		-- Include javadoc and junit snippets with java
		luasnip.filetype_extend("java", { "javadoc" })
		luasnip.filetype_extend("java", { "java-testing" })

		return {
			completion = {
				completeopt = "menu,menuone,preview,noselect",
			},
			snippet = { -- configure how nvim-cmp interacts with snippet engine
				expand = function(args)
					luasnip.lsp_expand(args.body)
				end,
			},
			mapping = {
				["<CR>"] = cmp.mapping(function(fallback)
					if cmp.visible() and cmp.get_active_entry() then
						cmp.confirm({
							select = true,
						})
					elseif cmp.visible() then
						cmp.close()
					else
						fallback()
					end
				end),

				["<C-k>"] = cmp.mapping.select_prev_item(),
				["<C-j>"] = cmp.mapping.select_next_item(),

				-- tab to scroll down in menu
				["<Tab>"] = cmp.mapping(function(fallback)
					if copilot.is_visible() then
						copilot.accept()
					elseif luasnip.locally_jumpable(1) then
						luasnip.jump(1)
					else
						neotab.tabout()
					end
				end, { "i", "s" }),

				-- -- shift+tab to scroll up in menu
				["<S-Tab>"] = cmp.mapping(function(fallback)
					if luasnip.locally_jumpable(-1) then
						luasnip.jump(-1)
					else
						fallback()
					end
				end, { "i", "s" }),
			},
			-- sources for autocompletion
			sources = cmp.config.sources({
				{ name = "nvim_lsp" },
				{ name = "luasnip" }, -- snippets
				{ name = "buffer" }, -- text within current buffer
				{ name = "path" }, -- file system paths
			}),

			-- configure lspkind for vs-code like pictograms in completion menu
			formatting = {
				fields = { "kind", "abbr", "menu" },
				format = function(entry, vim_item)
					local kind = lspkind.cmp_format({
						before = require("tailwind-tools.cmp").lspkind_format,
						maxwidth = 50,
						ellipsis_char = "...",
					})(entry, vim_item)

					local strings = vim.split(kind.kind, "%s", { trimempty = true })
					kind.kind = (strings[1] or "")
					kind.menu = "(" .. (strings[2] or "") .. ")"

					return kind
				end,
			},
            window = {
                completion = {
                    border = "rounded"
                },
                documentation = {
                    border = "rounded"
                }
            },
			cmp.setup.cmdline("/", {
				mapping = cmp.mapping.preset.cmdline(),
				sources = {
					{ name = "buffer" },
				},
			}),
			cmp.setup.cmdline(":", {
				mapping = cmp.mapping.preset.cmdline(),
				sources = cmp.config.sources({
					{ name = "path" },
				}, {
					{ name = "cmdline" },
				}),
				matching = { disallow_symbol_nonprefix_matching = false },
			}),
		}
	end,
}
