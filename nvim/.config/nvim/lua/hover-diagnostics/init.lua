local M = {}
local function hover_with_diagnostics()
	-- Try to peek folded lines under cursor
	local has_ufo, ufo = pcall(require, "ufo")
	if has_ufo then
		local winid = ufo.peekFoldedLinesUnderCursor()
		if winid then
			return
		end
	end

	local diagnostics = vim.diagnostic.get(0, { lnum = vim.api.nvim_win_get_cursor(0)[1] - 1 })
	local hover_text = {}
	local hover_ns = vim.api.nvim_create_namespace("lsp_hover_highlight")

	-- Add diagnostics at the top
	if #diagnostics > 0 then
		table.sort(diagnostics, function(a, b)
			return a.severity < b.severity
		end)
		for _, diagnostic in ipairs(diagnostics) do
			local severity = diagnostic.severity
			local name = vim.diagnostic.severity[severity]:lower()
			name = name:sub(1, 1):upper() .. name:sub(2)
			local sign = vim.diagnostic.config().signs.text[diagnostic.severity] or "?"
			table.insert(hover_text, string.format(" %s `%s` %s", sign, diagnostic.source, diagnostic.message))
		end
		table.insert(hover_text, "\n---")
	end

	-- Make the LSP request
	local params = vim.lsp.util.make_position_params(0, "utf-8")
	vim.lsp.buf_request_all(0, "textDocument/hover", params, function(results)
		local contents = {}
		local client_name_shown = false

		for client_id, response in pairs(results) do
			if response.err then
				vim.notify(response.err.message, vim.log.levels.WARN)
			elseif response.result then
				local client = vim.lsp.get_client_by_id(client_id)
				-- Show client name if multiple results
				if #vim.tbl_keys(results) > 1 and not client_name_shown then
					table.insert(contents, string.format("# %s", client.name))
					client_name_shown = true
				end

				local result = response.result
				-- Handle plaintext vs markdown content
				if type(result.contents) == "table" and result.contents.kind == "plaintext" then
					local lines = vim.split(result.contents.value or "", "\n", { trimempty = true })
					if #results == 1 then
						vim.list_extend(hover_text, lines)
					else
						table.insert(contents, "```")
						vim.list_extend(contents, lines)
						table.insert(contents, "```")
					end
				else
					vim.list_extend(contents, vim.lsp.util.convert_input_to_markdown_lines(result.contents))
				end

				-- Handle range highlighting
				if result.range then
					local start = result.range.start
					local end_ = result.range["end"]
					local start_idx = vim.lsp.util._get_line_byte_from_position(0, start, client.offset_encoding)
					local end_idx = vim.lsp.util._get_line_byte_from_position(0, end_, client.offset_encoding)

					vim.highlight.range(
						0,
						hover_ns,
						"LspReferenceText",
						{ start.line, start_idx },
						{ end_.line, end_idx },
						{ priority = vim.highlight.priorities.user }
					)
				end
			end
		end

		-- Extend hover_text with LSP contents
		vim.list_extend(hover_text, contents)

		if #hover_text == 0 then
			vim.notify("No information available")
			return
		end

		-- Show the floating window (pass empty filetype, set markdown after to avoid race condition)
		local bufnr, winnr = vim.lsp.util.open_floating_preview(hover_text, "", {
			border = "rounded",
			focus_id = "textDocument/hover",
			max_width = math.floor(vim.o.columns * 0.6),
		})

		vim.bo[bufnr].filetype = "markdown"

		if winnr then
			vim.wo[winnr].conceallevel = 3
			vim.wo[winnr].concealcursor = "ncv"
			vim.wo[winnr].signcolumn = "no"
		end

		if package.loaded["markview"] then
			-- Use strict_render for external plugins (handles conceal setup automatically)
			local strict_render = require("markview").strict_render
			strict_render:render(bufnr)

			-- Restart treesitter to apply conceal_lines for code fences
			vim.treesitter.stop(bufnr)
			vim.treesitter.start(bufnr)
		end

		-- Conceal the separator line vertically (requires Neovim 0.11+)
		if #diagnostics > 0 then
			local sep_line = #diagnostics
			vim.api.nvim_buf_set_extmark(bufnr, hover_ns, sep_line, 0, {
				conceal_lines = "",
			})
		end

		-- Resize window to account for concealed lines
		if winnr then
			local new_height = vim.api.nvim_win_text_height(winnr, { start_row = 0, end_row = -1 }).all
			local config = vim.api.nvim_win_get_config(winnr)
			config.height = new_height
			vim.api.nvim_win_set_config(winnr, config)
		end

		-- Add highlights for diagnostics
		if #diagnostics > 0 then
			local offset = 0
			for i, diagnostic in ipairs(diagnostics) do
				vim.api.nvim_buf_add_highlight(
					bufnr,
					-1,
					"DiagnosticSign" .. vim.diagnostic.severity[diagnostic.severity]:lower(),
					i - 1 + offset,
					0,
					2
				)
				offset = offset + #vim.split(diagnostic.message, "\n") - 1
			end
		end
	end)
end

M.setup = function()
	-- Setup on LspAttach
	vim.api.nvim_create_autocmd("LspAttach", {
		group = vim.api.nvim_create_augroup("HoverDiagnostics", { clear = true }),
		callback = function(ev)
			vim.keymap.set("n", "K", hover_with_diagnostics, {
				buffer = ev.buf,
				silent = true,
				desc = "Hover with diagnostics",
			})
		end,
	})
end

-- Export the function for manual use
M.hover = hover_with_diagnostics

return M
