# NeNe Docket のビルドと品質ゲート。
#
# 🔴 判定はここと xtask にある。CI は `make ci` を呼ぶだけで、CI 側にだけ存在する検査を作らない（QLT-005）。
# 🔴 道具の版は rust-toolchain.toml が決める。ここにも CI にも版を書かない（QLT-011）。
# 🔴 全件（make check）は既定ではない。差分に必要な検証を個別の target で選び、
#    全件は限定した検証で影響を覆えない理由を REASON= に添えたときだけ回す（QLT-001 / ADR 0003）。
CARGO ?= cargo

.PHONY: all check ci fmt fmt-check lint lint-windows build-windows test conformance validate-git coverage deny doc-check build prove screenshot doctor clean

all: doctor

## check — 全件。REASON が空なら実行前に拒む。
check:
	@if [ -z "$(REASON)" ]; then \
	  echo "QLT-001: 全件検証には理由が要る。例: make check REASON='共通の lint 集合を変えたので全クレートに影響する'"; \
	  exit 2; \
	fi
	@echo "全件検証: $(REASON)"
	$(MAKE) fmt-check lint test conformance deny doc-check build

## ci — CI の必須 check。Git 規約・PR の検証記録・差分の空白だけを見る（QLT-012）。
ci: validate-git
	git diff --check origin/main...HEAD

## fmt — rustfmt に設定ファイルを置かない（QLT-004）。
fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all --check

## lint — 規則との対応は docs/CODING_RULES.md。
## 🔴 --all-targets を外さないこと。外すとテストコードが検査対象から落ちる。
## 🔴 --locked は Cargo.lock が更新される状態でのゲート通過を拒む（QLT-011）。
lint:
	$(CARGO) clippy --workspace --all-targets --locked -- -D warnings

## lint-windows — Windows 版のクレートを Windows のターゲットで検査する（ADR 0002）。
## 🔴 Linux の make lint では Windows のコードは cfg で外れて見えない。Windows のコードはここでだけ検査される。
## X11 のクレートは Windows では組み立てない（cairo が無い）。合成ルートは Windows の画面ができるまで外す。
WINDOWS_TARGET := x86_64-pc-windows-gnu
WINDOWS_EXCLUDE := --exclude nene-docket-ui-x11 --exclude nene-docket
lint-windows:
	$(CARGO) clippy --workspace $(WINDOWS_EXCLUDE) --all-targets --locked --target $(WINDOWS_TARGET) -- -D warnings

## build-windows — Windows の exe を作る（mingw-w64 のリンカ）。target/x86_64-pc-windows-gnu/release/nene-docket.exe
build-windows:
	$(CARGO) build --release --locked -p nene-docket --target $(WINDOWS_TARGET)

test:
	$(CARGO) test --workspace --locked

## conformance — このリポジトリ固有の規約検査（CNF-0xx / xtask）。
conformance:
	$(CARGO) run --quiet -p xtask --locked -- conformance

## validate-git — GIT-001〜004。ローカルは origin/main..HEAD、CI は PR のメタデータ。
validate-git:
	$(CARGO) run --quiet -p xtask --locked -- validate-git

## coverage — 中核（core / application）のカバレッジ下限（QLT-009）。
## 🔴 分岐（--branch）は stable の rustc では測れない（-Z coverage-options=branch が要る。nene-ruler 2026-10-15 実測）。
##    下限は region に置く（nene-ruler ADR 0010 の継承）。下限を割ると理由を出さずに終了 1 で落ちる。
## 導入: cargo install cargo-llvm-cov --locked
coverage:
	@command -v cargo-llvm-cov >/dev/null || { echo "coverage: cargo-llvm-cov が無い。cargo install cargo-llvm-cov --locked"; exit 2; }
	$(CARGO) llvm-cov -p nene-docket-core -p nene-docket-application --locked --summary-only --fail-under-regions 90

## deny — 依存の許可制（ARC-013）。方針の正本は deny.toml。
## 🔴 --deny license-not-encountered を外さないこと。
## 導入: cargo install cargo-deny --locked
deny:
	@command -v cargo-deny >/dev/null || { echo "deny: cargo-deny が無い。cargo install cargo-deny --locked"; exit 2; }
	$(CARGO) deny --locked check --deny license-not-encountered

## doc-check — rustdoc の警告（壊れた intra-doc link 等）を失敗にする。
doc-check:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --workspace --no-deps --locked

build:
	$(CARGO) build --workspace --release --locked

## prove — ゲートが意図した違反で発火することを実測する（QLT-007）。結果は docs/quality/gate-proofs.md へ転記する。
prove:
	$(CARGO) run --quiet -p xtask --locked -- prove

## screenshot — Xvfb の上で窓を出し PNG を out/ に撮る（QLT-013）。
## 単体テストではなく環境検証である。結果は目で見て docs/quality/gate-proofs.md に記録する。
SCREENSHOT_DISPLAY ?= :99
screenshot:
	@command -v Xvfb >/dev/null || { echo "screenshot: Xvfb が無い（apt-get install xvfb）"; exit 2; }
	@command -v import >/dev/null || { echo "screenshot: ImageMagick が無い（apt-get install imagemagick）"; exit 2; }
	$(CARGO) build --locked -p nene-docket
	@mkdir -p out
	@Xvfb $(SCREENSHOT_DISPLAY) -screen 0 1024x640x24 >/dev/null 2>&1 & echo $$! > out/xvfb.pid; sleep 1; \
	  DISPLAY=$(SCREENSHOT_DISPLAY) ./target/debug/nene-docket & echo $$! > out/app.pid; sleep 2; \
	  DISPLAY=$(SCREENSHOT_DISPLAY) import -window root out/xvfb.png; \
	  kill $$(cat out/app.pid) $$(cat out/xvfb.pid) 2>/dev/null; rm -f out/*.pid
	@echo "out/xvfb.png を Read で見る"

## doctor — 開発環境（クラウドのコンテナ・WSLg）に道具が揃っているかを見る。検査ではない。
doctor:
	@rustc --version; cargo --version; cargo clippy --version; cargo fmt --version
	@command -v cargo-deny >/dev/null && cargo deny --version || echo "cargo-deny: 無い（cargo install cargo-deny --locked）"
	@command -v cargo-llvm-cov >/dev/null && cargo llvm-cov --version || echo "cargo-llvm-cov: 無い（cargo install cargo-llvm-cov --locked）"
	@pkg-config --modversion cairo >/dev/null 2>&1 && echo "cairo $$(pkg-config --modversion cairo)" || echo "cairo: 無い（apt-get install libcairo2-dev）"
	@command -v Xvfb >/dev/null && echo "Xvfb: ある" || echo "Xvfb: 無い（apt-get install xvfb）"
	@command -v import >/dev/null && echo "ImageMagick: ある" || echo "ImageMagick: 無い（apt-get install imagemagick）"
	@command -v xdotool >/dev/null && echo "xdotool: ある" || echo "xdotool: 無い（apt-get install xdotool）"
	@command -v x86_64-w64-mingw32-gcc >/dev/null && echo "mingw-w64: ある" || echo "mingw-w64: 無い（apt-get install mingw-w64。Windows の exe を組むのに要る・ADR 0002）"

clean:
	$(CARGO) clean
