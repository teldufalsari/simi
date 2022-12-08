from skimage.metrics import structural_similarity as ssim
import numpy as np
from PIL import Image


def get_rgb(path):
    image = Image.open(path)
    data = np.asarray(image)
    ch1, ch2, ch3 = map(lambda x: x[1][:, :, x[0]], enumerate([data] * 3))
    return ch1, ch2, ch3


def count_ssim_two_images(path_first, path_second):
    ch1_orig, ch2_orig, ch3_orig = get_rgb(path_first)
    ch1_emb, ch2_emb, ch3_emb = get_rgb(path_second)
    ssim_ch1, ssim_ch2, ssim_ch3 = ssim(ch1_orig, ch1_emb), ssim(ch2_orig, ch2_emb), ssim(ch3_orig, ch3_emb)
    write_table_line(ssim_ch1, ssim_ch2, ssim_ch3, get_pic_name(path_first))


def get_pic_name(path):
    return path.split("/")[-1]


def write_table_line(ssim_ch1, ssim_ch2, ssim_ch3, name):
    print("{:15} | {:10} | {:10} | {:10}".format(name, '%0.4f' % ssim_ch1, '%0.4f' % ssim_ch2, '%0.4f' % ssim_ch3))


def main():
    print("Embedded plain text")
    print("{:15} | {:10} | {:10} | {:10}".format("Name", "R", "G", "B"))
    count_ssim_two_images("../original/lena.png", "../embedded_plain_text/lena.png")
    count_ssim_two_images("../original/peppers.png", "../embedded_plain_text/peppers.png")
    count_ssim_two_images("../original/baboon.png", "../embedded_plain_text/baboon.png")
    count_ssim_two_images("../original/airplane.png", "../embedded_plain_text/airplane.png")
    count_ssim_two_images("../original/barbara.png", "../embedded_plain_text/barbara.png")
    count_ssim_two_images("../original/tiffany.png", "../embedded_plain_text/tiffany.png")
    count_ssim_two_images("../original/Zelda.png", "../embedded_plain_text/Zelda.png")

    print("Embedded aes ciphertext")
    print("{:15} | {:10} | {:10} | {:10}".format("Name", "R", "G", "B"))
    count_ssim_two_images("../original/lena.png", "../embedded_aes_ciphertext/lena.png")
    count_ssim_two_images("../original/peppers.png", "../embedded_aes_ciphertext/peppers.png")
    count_ssim_two_images("../original/baboon.png", "../embedded_aes_ciphertext/baboon.png")
    count_ssim_two_images("../original/airplane.png", "../embedded_aes_ciphertext/airplane.png")
    count_ssim_two_images("../original/barbara.png", "../embedded_aes_ciphertext/barbara.png")
    count_ssim_two_images("../original/tiffany.png", "../embedded_aes_ciphertext/tiffany.png")
    count_ssim_two_images("../original/Zelda.png", "../embedded_aes_ciphertext/Zelda.png")


if __name__ == "__main__":
    main()
