#!/usr/bin/env python3
# test_alchemy.py

import sys
import traceback

def main():
    try:
        import alchemy
    except Exception as e:
        print("❌ Failed to import 'alchemy' Python module.")
        traceback.print_exc()
        sys.exit(1)

    print("✅ Imported alchemy")

    # ---------- Utilities ----------
    try:
        raw = b"\x00\x01\xab\xcd"
        hx = alchemy.decode_hex_py("0001abcd")
        assert bytes(hx) == raw, f"decode_hex_py mismatch: {hx}"
        re_hx = alchemy.encode_hex_py(list(raw))
        assert re_hx.lower() == "0001abcd", f"encode_hex_py mismatch: {re_hx}"
        print("✅ Utilities: decode_hex_py / encode_hex_py")
    except Exception:
        print("❌ Utilities failed")
        traceback.print_exc()
        sys.exit(1)

    # ---------- Standardization ----------
    try:
        std_prefix = alchemy.PyStandardization("prefix")
        std_postfix = alchemy.PyStandardization("postfix")
        std_none    = alchemy.PyStandardization("none")
        print("✅ PyStandardization constructed (prefix/postfix/none)")
    except Exception:
        print("❌ PyStandardization construction failed")
        traceback.print_exc()
        sys.exit(1)

    # ---------- Generators: BTreeGen ----------
    try:
        # BTreeGen::from_config(size, freevar_prob, max_free_vars, std)
        bt = alchemy.PyBTreeGen.from_config(
            size=6,
            freevar_generation_probability=0.3,
            max_free_vars=3,
            std=std_prefix
        )
        one = bt.generate()
        many = bt.generate_n(5)
        assert isinstance(one, str) and len(one) > 0, "BTreeGen.generate must return string term"
        assert isinstance(many, list) and len(many) == 5 and all(isinstance(x, str) for x in many)
        print("✅ PyBTreeGen.generate / generate_n OK")
    except Exception:
        print("❌ PyBTreeGen tests failed")
        traceback.print_exc()
        sys.exit(1)

    # ---------- Soup: basic lifecycle ----------
    try:
        # Path 1: default config
        soup1 = alchemy.PySoup()
        assert soup1.len() == 0, "New soup should be empty"

        # Path 2: from reactor config
        reactor = alchemy.PyReactor()
        soup2 = alchemy.PySoup.from_config(reactor)
        assert soup2.len() == 0, "Soup.from_config should start empty"

        # Seed soup2 with 10 generated expressions from BTreeGen
        exprs = bt.generate_n(10)
        soup2.perturb(exprs)
        assert soup2.len() == 10, f"After perturb, expected 10 expressions, got {soup2.len()}"

        # Simulate a few steps, ensure the API responds
        steps = soup2.simulate_for(25, False)
        assert isinstance(steps, int), "simulate_for should return usize (int in Python)"

        # Read back data
        all_exprs = soup2.expressions()
        uniq = soup2.unique_expressions()
        counts = soup2.expression_counts()
        ent = soup2.population_entropy()
        col = soup2.collisions()
        ln = soup2.len()

        assert isinstance(all_exprs, list)
        assert isinstance(uniq, list)
        assert isinstance(counts, list) and all(
            isinstance(p, (tuple, list)) and len(p) == 2 and isinstance(p[0], str) and isinstance(p[1], int)
            for p in counts
        )
        assert isinstance(ent, float)
        assert isinstance(col, int)
        assert isinstance(ln, int)
        print(f"✅ PySoup lifecycle OK | len={ln}, uniq={len(uniq)}, collisions={col}, entropy={ent:.4f}")
    except Exception:
        print("❌ PySoup tests failed")
        traceback.print_exc()
        sys.exit(1)

    # ---------- FontanaGen (may legitimately return None) ----------
    try:
        fg = alchemy.PyFontanaGen.from_config(
            abs_range=(0.2, 0.6),
            app_range=(0.2, 0.6),
            max_depth=5,
            max_free_vars=3
        )
        maybe = fg.generate()
        # Current Rust stub returns None; accept either None or str
        assert (maybe is None) or isinstance(maybe, str)
        print(f"✅ PyFontanaGen.generate OK (returned {type(maybe).__name__})")
    except Exception:
        print("❌ PyFontanaGen tests failed")
        traceback.print_exc()
        sys.exit(1)

    print("\n🎉 All python.rs bindings exercised successfully.")

if __name__ == "__main__":
    main()
