import tensorflow as tf
import numpy as np
import librosa
import os

# Hyperparameters
SR = 16000  # Sampling rate
N_MFCC = 13  # Number of MFCCs
MAX_LEN = 160  # Maximum length of MFCC sequence


# Function to extract MFCC features
def extract_mfcc(file_path, max_len=MAX_LEN):
    y, sr = librosa.load(file_path, sr=SR)
    mfcc = librosa.feature.mfcc(y=y, sr=sr, n_mfcc=N_MFCC)
    if mfcc.shape[1] < max_len:
        pad_width = max_len - mfcc.shape[1]
        mfcc = np.pad(mfcc, pad_width=((0, 0), (0, pad_width)), mode="constant")
    else:
        mfcc = mfcc[:, :max_len]
    return mfcc.T


# Dataset preparation
TARGET_DIR = "data/target"  # Your voice samples
OTHER_DIR = "data/others"  # Other voices

X, y = [], []

# Load target user's voice samples
for file_name in os.listdir(TARGET_DIR):
    file_path = os.path.join(TARGET_DIR, file_name)
    X.append(extract_mfcc(file_path))
    y.append(1) 

# Load other voices
for file_name in os.listdir(OTHER_DIR):
    file_path = os.path.join(OTHER_DIR, file_name)
    X.append(extract_mfcc(file_path))
    y.append(0)

X = np.array(X)
y = np.array(y)


indices = np.arange(len(X))
np.random.shuffle(indices)
X = X[indices]
y = y[indices]

split = int(0.8 * len(X))
X_train, X_test = X[:split], X[split:]
y_train, y_test = y[:split], y[split:]

# Build the model
model = tf.keras.Sequential(
    [
        tf.keras.layers.Conv1D(64, 3, activation="relu", input_shape=(MAX_LEN, N_MFCC)),
        tf.keras.layers.MaxPooling1D(2),
        tf.keras.layers.Conv1D(128, 3, activation="relu"),
        tf.keras.layers.MaxPooling1D(2),
        tf.keras.layers.Flatten(),
        tf.keras.layers.Dense(128, activation="relu"),
        tf.keras.layers.Dense(1, activation="sigmoid"),
    ]
)

model.compile(optimizer="adam", loss="binary_crossentropy", metrics=["accuracy"])

# Train the model
model.fit(X_train, y_train, epochs=20, batch_size=16, validation_data=(X_test, y_test))

# Evaluate the model
eval_loss, eval_acc = model.evaluate(X_test, y_test)
print(f"Test Accuracy: {eval_acc * 100:.2f}%")

# Save the model
model.save("speaker_verification_model.h5")

# Prediction example
example_file = "example.wav"
example_mfcc = extract_mfcc(example_file)
example_mfcc = np.expand_dims(example_mfcc, axis=0)
prediction = model.predict(example_mfcc)
if prediction[0][0] > 0.5:
    print("This is your voice.")
else:
    print("This is not your voice.")
