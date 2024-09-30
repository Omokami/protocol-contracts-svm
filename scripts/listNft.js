function generatePassword(length) {
    if (length === void 0) { length = 12; }
    var charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+";
    var password = "";
    for (var i = 0; i < length; i++) {
        var randomIndex = Math.floor(Math.random() * charset.length);
        password += charset[randomIndex];
    }
    return password;
}
var passwordLength = process.argv[2] ? parseInt(process.argv[2]) : undefined;
var password = generatePassword(passwordLength);
console.log("Generated password: ".concat(password));
