#include "ast_converter.hpp"

#include <iostream>
#include <waveguide/intermediate/scope.hpp>

namespace waveguide {
namespace ast {

void ast_converter::operator()(int const&expr) const {
    data->current_value = int_literal(expr);
}

void ast_converter::operator()(float const&expr) const {
    data->current_value = float_literal(expr);
    std::cout << expr << " " << *data->current_value->data_as_float() << std::endl;
}

void ast_converter::operator()(bool const&expr) const {
    data->current_value = bool_literal(expr);
}

void ast_converter::operator()(signed_expression const&expr) const {
    recurse(expr.value);
    if (expr.sign == '-') {
        auto negate{std::make_shared<intr::command>(blt()->MUL)};
        negate->add_input(data->current_value);
        negate->add_input(int_literal(-1));
        auto output{std::make_shared<intr::value>(blt()->DEDUCE_LATER)};
        declare_temp_var(output);
        data->current_value = output;
        negate->add_output(output);
        add_command(negate);
    }
}

void ast_converter::operator()(variable_expression const&expr) const {
    data->current_value = lookup_var(expr.name);
    if (data->current_value == nullptr) {
        throw convert::ast_conversion_exception{
            "There is no variable in scope with the name '" + expr.name + "'."
        };
    }
    for (auto index_expr : expr.array_accesses) {
        auto output_value{std::make_shared<intr::value>(blt()->DEDUCE_LATER)};
        declare_temp_var(output_value);
        auto copy_command{
            std::make_shared<intr::command>(blt()->COPY_FROM_INDEX)
        };
        copy_command->add_input(data->current_value);
        recurse(index_expr);
        copy_command->add_input(data->current_value);
        copy_command->add_output(output_value);
        add_command(copy_command);
        data->current_value = output_value;
    }
}

void ast_converter::operator()(std::vector<expression> const&expr) const {
    auto copy_to{std::make_shared<intr::value>(
        std::make_shared<intr::array_data_type>(
            blt()->DEDUCE_LATER, expr.size()
        )
    )};
    declare_temp_var(copy_to);
    for (uint i = 0; i < expr.size(); i++) {
        recurse(expr[i]);
        auto insert{std::make_shared<intr::command>(blt()->COPY_TO_INDEX)};
        insert->add_input(data->current_value);
        insert->add_input(int_literal(i));
        insert->add_output(copy_to);
        add_command(insert);
    }
    data->current_value = copy_to;
}

void ast_converter::operator()(single_var_dec const&dec) const {
    recurse(dec.type);
    auto value{std::make_shared<intr::value>(data->current_type)};
    data->current_scope->declare_var(dec.name, value);
    data->current_value = value;
}

void ast_converter::operator()(function_expression const&expr) const {
    auto func = lookup_func(expr.function_name);
    if (func == nullptr) {
        throw convert::ast_conversion_exception{
            "There is no function in scope with the name '" 
                + expr.function_name + "'."
        };
    }
    auto command{std::make_shared<intr::command>(func)};
    for (auto const&input : expr.inputs) {
        recurse(input);
        command->add_input(data->current_value);
    }
    for (auto const&output : expr.outputs) {
        recurse(output);
        command->add_output(data->current_value);
    }
    add_command(command);
    for (auto const&lambda : expr.lambdas) {
        recurse(lambda);
    }
}

void ast_converter::operator()(operator_list_expression const&expr) const {
    recurse(expr.start_value);
    std::string last_op{""};
    bool join{false};
    intr::command_ptr last_command{nullptr};
    for (auto const&operation : expr.operations) {
        if (operation.op_char != last_op || !join) {
            if (last_command) {
                auto output{std::make_shared<intr::value>(blt()->DEDUCE_LATER)};
                declare_temp_var(output);
                last_command->add_output(output);
                add_command(last_command);
                data->current_value = output;
            }
            intr::scope_ptr func{nullptr};
            auto const&c = operation.op_char;
            if (c == "+" || c == "-") {
                func = blt()->ADD;
                join = true;
            } else if (c == "*" || c == "/") {
                func = blt()->MUL;
                join = true;
            } else if (c == "%") {
                func = blt()->MOD;
                join = false;
            } else if (c == ">=") {
                func = blt()->GTE;
                join = false;
            } else if (c == "<=") {
                func = blt()->LTE;
                join = false;
            } else if (c == ">") {
                func = blt()->GT;
                join = false;
            } else if (c == "<") {
                func = blt()->LT;
                join = false;
            } else if (c == "==") {
                func = blt()->EQ;
                join = false;
            } else if (c == "!=") {
                func = blt()->NEQ;
                join = false;
            } else if (c == "band") {
                func = blt()->BAND;
                join = true;
            } else if (c == "bor") {
                func = blt()->BOR;
                join = true;
            } else if (c == "bxor") {
                func = blt()->BXOR;
                join = true;
            } else if (c == "and") {
                func = blt()->AND;
                join = false;
            } else if (c == "or") {
                func = blt()->OR;
                join = false;
            } else if (c == "xor") {
                func = blt()->XOR;
                join = false;
            }
            last_command = std::make_shared<intr::command>(func);
            last_command->add_input(data->current_value);
        }
        recurse(operation.value);
        last_command->add_input(data->current_value);
    }
    if (last_command) {
        auto output{std::make_shared<intr::value>(blt()->DEDUCE_LATER)};
        declare_temp_var(output);
        last_command->add_output(output);
        add_command(last_command);
        data->current_value = output;
    }
}

}
}