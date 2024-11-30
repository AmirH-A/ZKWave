wave:
	mkdir -p circuit/temp/wave
	cd circuit && circom wave.circom --r1cs --wasm --sym --c
	mv circuit/wave_cpp circuit/temp/wave
	mv circuit/wave_js circuit/temp/wave

	mv circuit/temp/wave/wave_cpp/main.cpp circuit/temp/wave/wave_cpp/main.cpp.tmp
	python3 scripts/spit_output.py < circuit/temp/wave/wave_cpp/main.cpp.tmp > circuit/temp/wave/wave_cpp/main.cpp
	
	rm circuit/temp/wave/wave_cpp/main.cpp.tmp
	cd circuit/temp/wave/wave_cpp && make
	mv circuit/wave.r1cs circuit/temp/wave/wave.r1cs
	mv circuit/wave.sym circuit/temp/wave/wave.sym 

wave_zkey:
	cd circuit && snarkjs groth16 setup temp/wave/wave.r1cs 22_final.ptau wave_0000.zkey
	mv circuit/wave_0000.zkey circuit/temp/wave/wave_0000.zkey

wave_witness:
	cd circuit/temp/wave/wave_js
	node generate_witness.js wave.wasm ../../../input.json ../../../witness.wtns
	snarkjs wtns debug ../../../wave.sym ../../../witness.wtns ../../../input.json
clean:
	rm -rf circuit/*.r1cs circuit/*.wasm circuit/*.sym circuit/*.json circuit/*.wtns circuit/wave_cpp/ circuit/wave_js/  