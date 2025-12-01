Error executing callback:
...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:187: The coroutine failed with this message:
        context: cur_thread=main co_thread=<thread 0x010a522ca0> co_func=...are/nvim/lazy/diffview.nvim/lua/diffvi
ew/vcs/adapter.lua:350/diffview.nvim/lua/diffview/async.lua:187: The coroutine failed with this message:
        context: cur_thread=<thread 0x010a522ca0> co_thread=<thread 0x010a15e4e0> co_func=.../share/nvim/lazy/diff
...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:187: The coroutine failed with this message:
        context: cur_thread=<thread 0x010a15e4e0> co_thread=<thread 0x01097921f8> co_func=...re/nvim/lazy/diffview
...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:187: The coroutine failed with this message:
        context: cur_thread=<thread 0x01097921f8> co_thread=<thread 0x010ab6ccc0> co_func=...iffview.nvim/lua/diff
...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:187: The coroutine failed with this message:
        context: cur_thread=<thread 0x01097921f8> co_thread=<thread 0x0107d05828> co_func=...iffview.nvim/lua/diff
...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:187: The coroutine failed with this message:
        context: cur_thread=<thread 0x01097921f8> co_thread=<thread 0x010a378e78> co_func=...nvim/lua/diffview/sce
...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:187: The coroutine failed with this message:
        context: cur_thread=<thread 0x01097921f8> co_thread=<thread 0x010a37a290> co_func=...lazy/diffview.nvim/lu
...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:187: The coroutine failed with this message:
        context: cur_thread=<thread 0x01097921f8> co_thread=<thread 0x010a379160> co_func=...re/nvim/lazy/diffview
...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:187: The coroutine failed with this message:
        context: cur_thread=<thread 0x010a379160> co_thread=<thread 0x0109935f80> co_func=...re/nvim/lazy/diffview
.../share/nvim/lazy/diffview.nvim/lua/diffview/vcs/file.lua:303: E5560: nvim_buf_is_valid must not be called in a
stack traceback:xt
        [C]: in function 'nvim_buf_is_valid'
        .../share/nvim/lazy/diffview.nvim/lua/diffview/vcs/file.lua:303: in function 'is_valid'
        ...re/nvim/lazy/diffview.nvim/lua/diffview/scene/window.lua:95: in function <...re/nvim/lazy/diffview.nvim
        [C]: in function 'xpcall'>
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:361: in function <...cal/share/nvim/lazy/diffv
stack traceback:fview/async.lua:358>
        [C]: in function 'error'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:369: in function <...cal/share/nvim/lazy/diffv
stack traceback:fview/async.lua:358>
        [C]: in function 'error'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:187: in function 'raise'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:215: in function 'step'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:412: in function 'load_file'
        ...re/nvim/lazy/diffview.nvim/lua/diffview/scene/layout.lua:177: in function <...re/nvim/lazy/diffview.nvi
        [C]: in function 'xpcall'63>
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:361: in function <...cal/share/nvim/lazy/diffv
stack traceback:fview/async.lua:358>
        [C]: in function 'error'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:369: in function <...cal/share/nvim/lazy/diffv
stack traceback:fview/async.lua:358>
        [C]: in function 'error'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:187: in function 'raise'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:307: in function 'await'
        ...lazy/diffview.nvim/lua/diffview/scene/layouts/diff_3.lua:71: in function <...lazy/diffview.nvim/lua/dif
        [C]: in function 'xpcall'>
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:361: in function <...cal/share/nvim/lazy/diffv
stack traceback:fview/async.lua:358>
        [C]: in function 'error'
stack traceback:
        [C]: in function 'error'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:187: in function 'raise'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:307: in function 'await'
        ...iffview.nvim/lua/diffview/scene/views/diff/diff_view.lua:203: in function <...iffview.nvim/lua/diffview
/scene/views/diff/diff_view.lua:193>
        [C]: in function 'xpcall'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:361: in function <...cal/share/nvim/lazy/diffv
iew.nvim/lua/diffview/async.lua:358>
stack traceback:
        [C]: in function 'error'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:369: in function <...cal/share/nvim/lazy/diffv
iew.nvim/lua/diffview/async.lua:358>
stack traceback:
        [C]: in function 'error'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:187: in function 'raise'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:307: in function 'await'
        ...iffview.nvim/lua/diffview/scene/views/diff/diff_view.lua:278: in function 'func'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:373: in function <...cal/share/nvim/lazy/diffv
iew.nvim/lua/diffview/async.lua:358>
stack traceback:
        [C]: in function 'error'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:187: in function 'raise'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:215: in function 'step'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:247: in function 'notify_all'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:213: in function 'step'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:247: in function 'notify_all'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:213: in function 'step'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:247: in function 'notify_all'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:213: in function 'step'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:247: in function 'notify_all'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:213: in function 'step'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:247: in function 'notify_all'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:402: in function 'callback'
        ...re/nvim/lazy/diffview.nvim/lua/diffview/scene/window.lua:110: in function <...re/nvim/lazy/diffview.nvi
m/lua/diffview/scene/window.lua:92>
        [C]: in function 'xpcall'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:361: in function <...cal/share/nvim/lazy/diffv
iew.nvim/lua/diffview/async.lua:358>
stack traceback:
        [C]: in function 'error'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:369: in function <...cal/share/nvim/lazy/diffv
iew.nvim/lua/diffview/async.lua:358>
stack traceback:
        [C]: in function 'error'
        ...cal/share/nvim/lazy/diffview.nvim/lua/diffview/async.lua:187: in function 'raise'
